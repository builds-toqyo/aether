

use serde::{Serialize, Deserialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScopeData {

    pub waveform: WaveformData,

    pub vectorscope: VectorscopeData,

    pub histogram: HistogramData,

    pub metadata: ScopeMetadata,
}

impl ColorScopeData {

    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            waveform: WaveformData::new(resolution),
            vectorscope: VectorscopeData::new(resolution),
            histogram: HistogramData::new(resolution),
            metadata: ScopeMetadata::new(),
        }
    }


    pub fn clear(&mut self) {
        self.waveform.clear();
        self.vectorscope.clear();
        self.histogram.clear();
        self.metadata.reset();
    }


    pub fn size_bytes(&self) -> usize {
        self.waveform.size_bytes() + self.vectorscope.size_bytes() + self.histogram.size_bytes()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {

    pub luma: Vec<u8>,

    pub red: Vec<u8>,

    pub green: Vec<u8>,

    pub blue: Vec<u8>,

    pub config: WaveformConfig,
}

impl WaveformData {

    pub fn new(resolution: ScopeResolution) -> Self {
        let width = resolution.width();
        let height = resolution.height();

        Self {
            luma: vec![0; width * height],
            red: vec![0; width * height],
            green: vec![0; width * height],
            blue: vec![0; width * height],
            config: WaveformConfig::new(resolution),
        }
    }


    pub fn clear(&mut self) {
        self.luma.fill(0);
        self.red.fill(0);
        self.green.fill(0);
        self.blue.fill(0);
    }


    pub fn channel_data(&self, channel: WaveformChannel) -> &[u8] {
        match channel {
            WaveformChannel::Luma => &self.luma,
            WaveformChannel::Red => &self.red,
            WaveformChannel::Green => &self.green,
            WaveformChannel::Blue => &self.blue,
        }
    }


    pub fn channel_data_mut(&mut self, channel: WaveformChannel) -> &mut [u8] {
        match channel {
            WaveformChannel::Luma => &mut self.luma,
            WaveformChannel::Red => &mut self.red,
            WaveformChannel::Green => &mut self.green,
            WaveformChannel::Blue => &mut self.blue,
        }
    }


    pub fn set_pixel(&mut self, x: usize, y: usize, channel: WaveformChannel, value: u8) {
        let width = self.config.resolution.width();
        if x < width && y < self.config.resolution.height() {
            let index = y * width + x;
            self.channel_data_mut(channel)[index] = value;
        }
    }


    pub fn size_bytes(&self) -> usize {
        self.luma.len() + self.red.len() + self.green.len() + self.blue.len()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorscopeData {

    pub points: Vec<VectorscopePoint>,

    pub intensity_grid: Vec<u16>,

    pub config: VectorscopeConfig,
}

impl VectorscopeData {

    pub fn new(resolution: ScopeResolution) -> Self {
        let grid_size = resolution.vectorscope_grid_size();

        Self {
            points: Vec::new(),
            intensity_grid: vec![0; grid_size * grid_size],
            config: VectorscopeConfig::new(resolution),
        }
    }


    pub fn clear(&mut self) {
        self.points.clear();
        self.intensity_grid.fill(0);
    }


    pub fn add_point(&mut self, u: f32, v: f32, intensity: u16) {
        let point = VectorscopePoint::new(u, v, intensity);
        self.points.push(point);


        let grid_x = ((u + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;
        let grid_y = ((v + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;

        if grid_x < self.config.resolution.vectorscope_grid_size() &&
           grid_y < self.config.resolution.vectorscope_grid_size() {
            let index = grid_y * self.config.resolution.vectorscope_grid_size() + grid_x;
            self.intensity_grid[index] = self.intensity_grid[index].saturating_add(intensity);
        }
    }


    pub fn get_intensity(&self, u: f32, v: f32) -> u16 {
        let grid_x = ((u + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;
        let grid_y = ((v + 0.5) * self.config.resolution.vectorscope_grid_size() as f32) as usize;

        if grid_x < self.config.resolution.vectorscope_grid_size() &&
           grid_y < self.config.resolution.vectorscope_grid_size() {
            let index = grid_y * self.config.resolution.vectorscope_grid_size() + grid_x;
            self.intensity_grid[index]
        } else {
            0
        }
    }


    pub fn size_bytes(&self) -> usize {
        self.points.len() * std::mem::size_of::<VectorscopePoint>() +
        self.intensity_grid.len() * std::mem::size_of::<u16>()
    }
}


#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VectorscopePoint {

    pub u: f32,

    pub v: f32,

    pub intensity: u16,
}

impl VectorscopePoint {

    pub fn new(u: f32, v: f32, intensity: u16) -> Self {
        Self {
            u: u.clamp(-0.5, 0.5),
            v: v.clamp(-0.5, 0.5),
            intensity,
        }
    }


    pub fn from_rgb(r: f32, g: f32, b: f32) -> Self {

        let y = 0.299 * r + 0.587 * g + 0.114 * b;
        let u = (b - y) / 1.772;
        let v = (r - y) / 1.402;

        Self::new(u, v, 1)
    }
}


#[derive(Debug, Clone)]
pub struct HistogramData {

    pub red: [u32; 256],

    pub green: [u32; 256],

    pub blue: [u32; 256],

    pub luma: [u32; 256],

    pub config: HistogramConfig,
}

impl serde::Serialize for HistogramData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        
        let mut state = serializer.serialize_struct("HistogramData", 5)?;
        state.serialize_field("red", &self.red.to_vec())?;
        state.serialize_field("green", &self.green.to_vec())?;
        state.serialize_field("blue", &self.blue.to_vec())?;
        state.serialize_field("luma", &self.luma.to_vec())?;
        state.serialize_field("config", &self.config)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for HistogramData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct HistogramDataHelper {
            red: Vec<u32>,
            green: Vec<u32>,
            blue: Vec<u32>,
            luma: Vec<u32>,
            config: HistogramConfig,
        }

        let helper = HistogramDataHelper::deserialize(deserializer)?;
        
        let red: [u32; 256] = helper.red.try_into().unwrap_or_else(|_| [0; 256]);
        let green: [u32; 256] = helper.green.try_into().unwrap_or_else(|_| [0; 256]);
        let blue: [u32; 256] = helper.blue.try_into().unwrap_or_else(|_| [0; 256]);
        let luma: [u32; 256] = helper.luma.try_into().unwrap_or_else(|_| [0; 256]);

        Ok(HistogramData {
            red,
            green,
            blue,
            luma,
            config: helper.config,
        })
    }
}

impl HistogramData {

    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            red: [0; 256],
            green: [0; 256],
            blue: [0; 256],
            luma: [0; 256],
            config: HistogramConfig::new(resolution),
        }
    }


    pub fn clear(&mut self) {
        self.red.fill(0);
        self.green.fill(0);
        self.blue.fill(0);
        self.luma.fill(0);
    }


    pub fn add_sample(&mut self, r: u8, g: u8, b: u8) {
        self.red[r as usize] += 1;
        self.green[g as usize] += 1;
        self.blue[b as usize] += 1;


        let y = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
        self.luma[y as usize] += 1;
    }


    pub fn channel_data(&self, channel: HistogramChannel) -> &[u32; 256] {
        match channel {
            HistogramChannel::Red => &self.red,
            HistogramChannel::Green => &self.green,
            HistogramChannel::Blue => &self.blue,
            HistogramChannel::Luma => &self.luma,
        }
    }


    pub fn max_value(&self, channel: HistogramChannel) -> u32 {
        self.channel_data(channel).iter().copied().max().unwrap_or(0)
    }


    pub fn normalize(&self, channel: HistogramChannel) -> Vec<u8> {
        let data = self.channel_data(channel);
        let max_val = self.max_value(channel);

        if max_val == 0 {
            return vec![0; 256];
        }

        data.iter()
            .map(|&val| ((val as f32 / max_val as f32) * 255.0) as u8)
            .collect()
    }


    pub fn size_bytes(&self) -> usize {
        4 * 256 * std::mem::size_of::<u32>()
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeMetadata {

    pub frame_number: u64,

    pub timestamp: f64,

    pub source_resolution: (u32, u32),

    pub color_space: ColorSpace,

    pub range: VideoRange,

    pub stats: ScopeStats,
}

impl ScopeMetadata {

    pub fn new() -> Self {
        Self {
            frame_number: 0,
            timestamp: 0.0,
            source_resolution: (1920, 1080),
            color_space: ColorSpace::Rec709,
            range: VideoRange::Limited,
            stats: ScopeStats::new(),
        }
    }


    pub fn reset(&mut self) {
        self.frame_number = 0;
        self.timestamp = 0.0;
        self.stats = ScopeStats::new();
    }


    pub fn update_frame(&mut self, frame_number: u64, timestamp: f64) {
        self.frame_number = frame_number;
        self.timestamp = timestamp;
        self.stats.frames_processed += 1;
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeStats {

    pub frames_processed: u64,

    pub avg_processing_time_us: f64,

    pub peak_processing_time_us: f64,

    pub total_pixels_processed: u64,
}

impl ScopeStats {

    pub fn new() -> Self {
        Self {
            frames_processed: 0,
            avg_processing_time_us: 0.0,
            peak_processing_time_us: 0.0,
            total_pixels_processed: 0,
        }
    }


    pub fn update_processing_time(&mut self, processing_time_us: f64, pixels_processed: u64) {
        self.total_pixels_processed += pixels_processed;

        if processing_time_us > self.peak_processing_time_us {
            self.peak_processing_time_us = processing_time_us;
        }


        if self.frames_processed > 0 {
            self.avg_processing_time_us =
                (self.avg_processing_time_us * (self.frames_processed - 1) as f64 + processing_time_us) /
                self.frames_processed as f64;
        } else {
            self.avg_processing_time_us = processing_time_us;
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeResolution {
    Low,
    Medium,
    High,
    Ultra,
}

impl ScopeResolution {

    pub fn width(&self) -> usize {
        match self {
            ScopeResolution::Low => 640,
            ScopeResolution::Medium => 1280,
            ScopeResolution::High => 1920,
            ScopeResolution::Ultra => 3840,
        }
    }


    pub fn height(&self) -> usize {
        match self {
            ScopeResolution::Low => 480,
            ScopeResolution::Medium => 720,
            ScopeResolution::High => 1080,
            ScopeResolution::Ultra => 2160,
        }
    }


    pub fn vectorscope_grid_size(&self) -> usize {
        match self {
            ScopeResolution::Low => 256,
            ScopeResolution::Medium => 512,
            ScopeResolution::High => 1024,
            ScopeResolution::Ultra => 2048,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveformChannel {
    Luma,
    Red,
    Green,
    Blue,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistogramChannel {
    Red,
    Green,
    Blue,
    Luma,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorSpace {
    Rec601,
    Rec709,
    Rec2020,
    DCIP3,
    SRGB,
    ProPhotoRGB,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VideoRange {
    Limited,
    Full,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformConfig {
    pub resolution: ScopeResolution,
    pub mode: WaveformMode,
    pub scale: WaveformScale,
    pub filtering: bool,
}

impl WaveformConfig {
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            resolution,
            mode: WaveformMode::Parade,
            scale: WaveformScale::IRE,
            filtering: true,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveformMode {
    Luma,
    Parade,
    Overlay,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaveformScale {
    IRE,
    Millivolts,
    Bits,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorscopeConfig {
    pub resolution: ScopeResolution,
    pub targets: Vec<VectorscopeTarget>,
    pub scale: VectorscopeScale,
    pub filtering: bool,
}

impl VectorscopeConfig {
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            resolution,
            targets: vec![
                VectorscopeTarget::Primary,
                VectorscopeTarget::SkinTones,
                VectorscopeTarget::Blue,
                VectorscopeTarget::Yellow,
                VectorscopeTarget::Cyan,
                VectorscopeTarget::Green,
                VectorscopeTarget::Magenta,
                VectorscopeTarget::Red,
            ],
            scale: VectorscopeScale::UV,
            filtering: true,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VectorscopeTarget {
    Primary,
    SkinTones,
    Blue,
    Yellow,
    Cyan,
    Green,
    Magenta,
    Red,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorscopeScale {
    UV,
    YPbPr,
    XY,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramConfig {
    pub resolution: ScopeResolution,
    pub mode: HistogramMode,
    pub log_scale: bool,
    pub show_peaks: bool,
}

impl HistogramConfig {
    pub fn new(resolution: ScopeResolution) -> Self {
        Self {
            resolution,
            mode: HistogramMode::RGB,
            log_scale: false,
            show_peaks: true,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HistogramMode {
    RGB,
    Luma,
    Individual,
    Parade,
}
