use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::engine::rendering::formats::{VideoFormat, AudioFormat};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EncoderPreset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
    Placebo,
}

impl EncoderPreset {
    pub fn to_ffmpeg_name(&self) -> &'static str {
        match self {
            EncoderPreset::UltraFast => "ultrafast",
            EncoderPreset::SuperFast => "superfast",
            EncoderPreset::VeryFast => "veryfast",
            EncoderPreset::Faster => "faster",
            EncoderPreset::Fast => "fast",
            EncoderPreset::Medium => "medium",
            EncoderPreset::Slow => "slow",
            EncoderPreset::Slower => "slower",
            EncoderPreset::VerySlow => "veryslow",
            EncoderPreset::Placebo => "placebo",
        }
    }
    
    /// Get a human-readable description for this preset
    pub fn description(&self) -> &'static str {
        match self {
            EncoderPreset::UltraFast => "Fastest encoding, lowest quality",
            EncoderPreset::SuperFast => "Very fast encoding, lower quality",
            EncoderPreset::VeryFast => "Fast encoding, low quality",
            EncoderPreset::Faster => "Fast encoding, decent quality",
            EncoderPreset::Fast => "Quick encoding, good quality",
            EncoderPreset::Medium => "Balanced speed and quality",
            EncoderPreset::Slow => "Slow encoding, high quality",
            EncoderPreset::Slower => "Very slow encoding, higher quality",
            EncoderPreset::VerySlow => "Extremely slow encoding, best quality",
            EncoderPreset::Placebo => "Painfully slow encoding, marginally better quality",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncoderOptions {
    pub video_format: VideoFormat,
    
    pub audio_format: AudioFormat,
    
    pub preset: EncoderPreset,
    
    pub crf: u8,
    
    pub video_bitrate: u32,
    
    pub audio_bitrate: u32,
    
    pub two_pass: bool,
    
    pub hardware_acceleration: bool,
    
    pub additional_options: HashMap<String, String>,
}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self {
            video_format: VideoFormat::H264,
            audio_format: AudioFormat::Aac,
            preset: EncoderPreset::Medium,
            crf: 23,
            video_bitrate: 0,
            audio_bitrate: 128000,
            two_pass: false,
            hardware_acceleration: false,
            additional_options: HashMap::new(),
        }
    }
}

impl EncoderOptions {
    pub fn new(video_format: VideoFormat, audio_format: AudioFormat) -> Self {
        Self {
            video_format,
            audio_format,
            ..Default::default()
        }
    }
    
    pub fn high_quality() -> Self {
        Self {
            video_format: VideoFormat::H265,
            audio_format: AudioFormat::Flac,
            preset: EncoderPreset::Slow,
            crf: 18,
            video_bitrate: 0,
            audio_bitrate: 320000,
            two_pass: true,
            hardware_acceleration: false,
            additional_options: HashMap::new(),
        }
    }
    
    pub fn web_delivery() -> Self {
        Self {
            video_format: VideoFormat::H264,
            audio_format: AudioFormat::Aac,
            preset: EncoderPreset::Medium,
            crf: 23,
            video_bitrate: 0,
            audio_bitrate: 128000,
            two_pass: false,
            hardware_acceleration: false,
            additional_options: HashMap::new(),
        }
    }
    
    pub fn fast_preview() -> Self {
        Self {
            video_format: VideoFormat::H264,
            audio_format: AudioFormat::Aac,
            preset: EncoderPreset::UltraFast,
            crf: 28,
            video_bitrate: 0,
            audio_bitrate: 96000,
            two_pass: false,
            hardware_acceleration: true,
            additional_options: HashMap::new(),
        }
    }
    
    pub fn professional() -> Self {
        Self {
            video_format: VideoFormat::ProRes,
            audio_format: AudioFormat::Pcm,
            preset: EncoderPreset::Medium,
            crf: 0,
            video_bitrate: 100000000, // 100 Mbps
            audio_bitrate: 1536000,   // 1.5 Mbps
            two_pass: false,
            hardware_acceleration: false,
            additional_options: {
                let mut options = HashMap::new();
                options.insert("profile:v".to_string(), "3".to_string()); // ProRes HQ
                options
            },
        }
    }
    
    pub fn add_option(&mut self, key: &str, value: &str) -> &mut Self {
        self.additional_options.insert(key.to_string(), value.to_string());
        self
    }
    
    pub fn with_preset(&mut self, preset: EncoderPreset) -> &mut Self {
        self.preset = preset;
        self
    }
    
    pub fn with_crf(&mut self, crf: u8) -> &mut Self {
        self.crf = crf;
        self
    }
    
    pub fn with_video_bitrate(&mut self, bitrate: u32) -> &mut Self {
        self.video_bitrate = bitrate;
        self
    }
    
    pub fn with_audio_bitrate(&mut self, bitrate: u32) -> &mut Self {
        self.audio_bitrate = bitrate;
        self
    }
    
    pub fn with_two_pass(&mut self, enabled: bool) -> &mut Self {
        self.two_pass = enabled;
        self
    }
    
    pub fn with_hardware_acceleration(&mut self, enabled: bool) -> &mut Self {
        self.hardware_acceleration = enabled;
        self
    }
    
    pub fn to_ffmpeg_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        
        let codec_name = if self.hardware_acceleration {
            match self.video_format {
                VideoFormat::H264 => "h264_videotoolbox", // For macOS
                VideoFormat::H265 => "hevc_videotoolbox", // For macOS
                _ => self.video_format.to_ffmpeg_name(),
            }
        } else {
            self.video_format.to_ffmpeg_name()
        };
        
        args.push("-c:v".to_string());
        args.push(codec_name.to_string());
        
        args.push("-c:a".to_string());
        args.push(self.audio_format.to_ffmpeg_name().to_string());
        
        if matches!(self.video_format, VideoFormat::H264 | VideoFormat::H265) {
            args.push("-preset".to_string());
            args.push(self.preset.to_ffmpeg_name().to_string());
        }
        
        if self.video_bitrate == 0 {
            if matches!(self.video_format, VideoFormat::H264 | VideoFormat::H265 | VideoFormat::Vp9) {
                args.push("-crf".to_string());
                args.push(self.crf.to_string());
            }
        } else {
            args.push("-b:v".to_string());
            args.push(format!("{}k", self.video_bitrate / 1000));
        }
        
        args.push("-b:a".to_string());
        args.push(format!("{}k", self.audio_bitrate / 1000));
        
        if self.two_pass {
            args.push("-pass".to_string());
            args.push("1".to_string());
        }
        
        for (key, value) in &self.additional_options {
            args.push(format!("-{}", key));
            args.push(value.clone());
        }
        
        args
    }
}
