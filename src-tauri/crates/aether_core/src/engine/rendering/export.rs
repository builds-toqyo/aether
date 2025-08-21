use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::Result;
use ffmpeg_next as ffmpeg;
use crate::engine::editing::types::EditingError;
use crate::engine::rendering::formats::{VideoFormat, AudioFormat, ContainerFormat};
use crate::engine::rendering::encoder::EncoderPreset;

pub type ExportCallback = Arc<Mutex<dyn Fn(ExportProgress) + Send + 'static>>;

#[derive(Debug, Clone)]
pub struct ExportOptions {
    pub input_path: PathBuf,
    
    pub output_path: PathBuf,
    
    pub container_format: ContainerFormat,
    
    pub video_format: VideoFormat,
    
    pub audio_format: AudioFormat,
    
    pub video_bitrate: u32,
    
    pub audio_bitrate: u32,
    
    pub frame_rate: f64,
    
    pub width: u32,
    
    pub height: u32,
    
    pub encoder_preset: EncoderPreset,
    
    pub crf: u8,
    
    pub hardware_acceleration: bool,
    
    pub threads: u8,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            input_path: PathBuf::new(),
            output_path: PathBuf::new(),
            container_format: ContainerFormat::Mp4,
            video_format: VideoFormat::H264,
            audio_format: AudioFormat::Aac,
            video_bitrate: 0,
            audio_bitrate: 0,
            frame_rate: 0.0,
            width: 0,
            height: 0,
            encoder_preset: EncoderPreset::Medium,
            crf: 23,
            hardware_acceleration: false,
            threads: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportProgress {
    pub current_frame: u64,
    
    pub total_frames: u64,
    
    pub current_time: f64,
    
    pub total_duration: f64,
    
    pub percent: f64,
    
    pub complete: bool,
    
    pub error: Option<String>,
}

pub struct Exporter {
    options: ExportOptions,
    
    progress: Arc<Mutex<ExportProgress>>,
    
    progress_callback: Option<ExportCallback>,
    
    export_thread: Option<thread::JoinHandle<Result<(), EditingError>>>,
    
    cancel_flag: Arc<Mutex<bool>>,
}

impl Exporter {
    pub fn new(options: ExportOptions) -> Result<Self, EditingError> {
        ffmpeg::init().map_err(|e| EditingError::ExportError(format!("Failed to initialize FFmpeg: {}", e)))?;
        
        let progress = Arc::new(Mutex::new(ExportProgress {
            current_frame: 0,
            total_frames: 0,
            current_time: 0.0,
            total_duration: 0.0,
            percent: 0.0,
            complete: false,
            error: None,
        }));
        
        Ok(Self {
            options,
            progress,
            progress_callback: None,
            export_thread: None,
            cancel_flag: Arc::new(Mutex::new(false)),
        })
    }
    
    pub fn set_progress_callback<F>(&mut self, callback: F)
    where
        F: Fn(ExportProgress) + Send + 'static,
    {
        self.progress_callback = Some(Arc::new(Mutex::new(callback)));
    }
    
    pub fn start_export(&mut self) -> Result<(), EditingError> {
        *self.cancel_flag.lock().unwrap() = false;
        
        let options = self.options.clone();
        let progress = self.progress.clone();
        let callback = self.progress_callback.clone();
        let cancel_flag = self.cancel_flag.clone();
        
        let handle = thread::spawn(move || {
            let input_path = options.input_path.to_string_lossy().to_string();
            let mut input_context = match ffmpeg::format::input(&input_path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    let error_msg = format!("Failed to open input file: {}", e);
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    return Err(EditingError::ExportError(error_msg));
                }
            };
            
            if let Err(e) = input_context.dump() {
                let error_msg = format!("Failed to read stream information: {}", e);
                Self::update_progress_with_error(&progress, &callback, &error_msg);
                return Err(EditingError::ExportError(error_msg));
            }
            
            let (video_stream_index, audio_stream_index) = {
                let video_stream = input_context.streams()
                    .best(ffmpeg::media::Type::Video)
                    .map(|s| s.index());
                
                let audio_stream = input_context.streams()
                    .best(ffmpeg::media::Type::Audio)
                    .map(|s| s.index());
                
                (video_stream, audio_stream)
            };
            
            let (width, height, frame_rate, total_frames, duration) = if let Some(stream_index) = video_stream_index {
                let stream = input_context.stream(stream_index).unwrap();
                let codec_context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
                
                let width = codec_context.width();
                let height = codec_context.height();
                
                let frame_rate = if let Some(rate) = stream.avg_frame_rate() {
                    rate.numerator() as f64 / rate.denominator() as f64
                } else {
                    25.0 // Default frame rate
                };
                
                let duration = stream.duration() as f64 * f64::from(stream.time_base());
                let total_frames = (duration * frame_rate) as u64;
                
                (width, height, frame_rate, total_frames, duration)
            } else {
                let error_msg = "No video stream found in input file".to_string();
                Self::update_progress_with_error(&progress, &callback, &error_msg);
                return Err(EditingError::ExportError(error_msg));
            };
            
            {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.total_frames = total_frames;
                progress_guard.total_duration = duration;
                
                if let Some(callback) = &callback {
                    callback.lock().unwrap()(progress_guard.clone());
                }
            }
            
            let output_path = options.output_path.to_string_lossy().to_string();
            let mut output_context = match ffmpeg::format::output(&output_path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    let error_msg = format!("Failed to create output file: {}", e);
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    return Err(EditingError::ExportError(error_msg));
                }
            };
            
            let format_name = options.container_format.to_ffmpeg_name();
            output_context.set_format(format_name);
            
            let video_codec_name = options.video_format.to_ffmpeg_name();
            let video_codec = ffmpeg::encoder::find_by_name(video_codec_name)
                .ok_or_else(|| {
                    let error_msg = format!("Video codec not found: {}", video_codec_name);
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    EditingError::ExportError(error_msg)
                })?;
            
            let mut video_stream = output_context.add_stream(video_codec)?;
            
            {
                let mut encoder = video_stream.codec().encoder().video()?;
                
                let out_width = if options.width > 0 { options.width } else { width as u32 };
                let out_height = if options.height > 0 { options.height } else { height as u32 };
                encoder.set_width(out_width);
                encoder.set_height(out_height);
                
                encoder.set_format(ffmpeg::format::pixel::Pixel::YUV420P);
                
                let out_frame_rate = if options.frame_rate > 0.0 { options.frame_rate } else { frame_rate };
                let frame_rate_rational = ffmpeg::util::rational::Rational::new(
                    (out_frame_rate * 1000.0) as i32,
                    1000,
                );
                encoder.set_time_base(frame_rate_rational.invert());
                video_stream.set_time_base(frame_rate_rational.invert());
                
                if options.video_bitrate > 0 {
                    encoder.set_bit_rate(options.video_bitrate as i64);
                } else {
                    encoder.set_option("crf", &options.crf.to_string())?;
                }
                
                encoder.set_option("preset", options.encoder_preset.to_ffmpeg_name())?;
                
                if options.threads > 0 {
                    encoder.set_option("threads", &options.threads.to_string())?;
                }
                
                encoder.open()?;
            }
            
            let mut audio_stream_index_out = None;
            if let Some(audio_index) = audio_stream_index {
                let audio_codec_name = options.audio_format.to_ffmpeg_name();
                let audio_codec = ffmpeg::encoder::find_by_name(audio_codec_name)
                    .ok_or_else(|| {
                        let error_msg = format!("Audio codec not found: {}", audio_codec_name);
                        Self::update_progress_with_error(&progress, &callback, &error_msg);
                        EditingError::ExportError(error_msg)
                    })?;
                
                let mut audio_stream = output_context.add_stream(audio_codec)?;
                audio_stream_index_out = Some(audio_stream.index());
                
                {
                    let input_stream = input_context.stream(audio_index).unwrap();
                    let input_codec_context = ffmpeg::codec::context::Context::from_parameters(input_stream.parameters())?;
                    let input_codec_par = input_codec_context.parameters();
                    
                    let mut encoder = audio_stream.codec().encoder().audio()?;
                    
                    encoder.set_rate(input_codec_par.rate() as i32);
                    encoder.set_channels(input_codec_par.channels() as i32);
                    encoder.set_channel_layout(input_codec_par.channel_layout());
                    encoder.set_format(ffmpeg::format::sample::Sample::F32(ffmpeg::format::sample::Type::Planar));
                    
                    let time_base = ffmpeg::util::rational::Rational::new(1, input_codec_par.rate() as i32);
                    encoder.set_time_base(time_base);
                    audio_stream.set_time_base(time_base);
                    
                    if options.audio_bitrate > 0 {
                        encoder.set_bit_rate(options.audio_bitrate as i64);
                    }
                    
                    encoder.open()?;
                }
            }
            
            output_context.write_header()?;
            
            let mut video_decoder = {
                let stream = input_context.stream(video_stream_index.unwrap()).unwrap();
                let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
                context.decoder().video()?
            };
            
            let mut audio_decoder = if let Some(audio_index) = audio_stream_index {
                let stream = input_context.stream(audio_index).unwrap();
                let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
                Some(context.decoder().audio()?)
            } else {
                None
            };
            
            let mut scaler = {
                let out_width = if options.width > 0 { options.width } else { width as u32 };
                let out_height = if options.height > 0 { options.height } else { height as u32 };
                
                ffmpeg::software::scaling::context::Context::get(
                    video_decoder.format(),
                    video_decoder.width(),
                    video_decoder.height(),
                    ffmpeg::format::pixel::Pixel::YUV420P,
                    out_width,
                    out_height,
                    ffmpeg::software::scaling::flag::Flags::BILINEAR,
                )?
            };
            
            let mut resampler = if let Some(ref audio_decoder) = audio_decoder {
                let out_stream = output_context.stream(audio_stream_index_out.unwrap()).unwrap();
                let out_codec = out_stream.codec();
                let out_codec_context = out_codec.encoder().audio()?;
                
                Some(ffmpeg::software::resampling::context::Context::get(
                    audio_decoder.format(),
                    audio_decoder.channel_layout(),
                    audio_decoder.rate(),
                    ffmpeg::format::sample::Sample::F32(ffmpeg::format::sample::Type::Planar),
                    out_codec_context.channel_layout(),
                    out_codec_context.rate(),
                )?)
            } else {
                None
            };
            
            let mut decoded = ffmpeg::frame::Video::empty();
            let mut audio_decoded = ffmpeg::frame::Audio::empty();
            let mut encoded = ffmpeg::frame::Video::empty();
            let mut audio_encoded = ffmpeg::frame::Audio::empty();
            let mut packet = ffmpeg::packet::Packet::empty();
            
            let mut frame_count = 0;
            
            while let Ok(true) = input_context.read(&mut packet) {
                if *cancel_flag.lock().unwrap() {
                    let error_msg = "Export cancelled".to_string();
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    return Err(EditingError::ExportError(error_msg));
                }
                
=                if let Some(stream_index) = video_stream_index {
                    if packet.stream() == stream_index {
                        video_decoder.send_packet(&packet)?;
                        
                        while video_decoder.receive_frame(&mut decoded).is_ok() {
                            scaler.run(&decoded, &mut encoded)?;
                            
                            let time_base = input_context.stream(stream_index).unwrap().time_base();
                            let pts = packet.pts().unwrap_or(ffmpeg::util::format::Rational::new(0, 1));
                            let pts_seconds = pts.numerator() as f64 * f64::from(time_base) / pts.denominator() as f64;
                            
                            encoded.set_pts(Some(frame_count as i64));
                            
                            let out_stream = output_context.stream(0).unwrap();
                            let mut out_codec = out_stream.codec();
                            let mut encoder = out_codec.encoder().video()?;
                            
                            encoder.send_frame(&encoded)?;
                            
                            let mut out_packet = ffmpeg::packet::Packet::empty();
                            while encoder.receive_packet(&mut out_packet).is_ok() {
                                out_packet.set_stream(0);
                                out_packet.rescale_ts(
                                    encoder.time_base(),
                                    out_stream.time_base(),
                                );
                                
                                output_context.write_packet(&out_packet)?;
                            }
                            
                            frame_count += 1;
                            {
                                let mut progress_guard = progress.lock().unwrap();
                                progress_guard.current_frame = frame_count;
                                progress_guard.current_time = pts_seconds;
                                progress_guard.percent = (frame_count as f64 / total_frames as f64) * 100.0;
                                
                                if let Some(callback) = &callback {
                                    callback.lock().unwrap()(progress_guard.clone());
                                }
                            }
                        }
                    }
                }
                
                if let Some(audio_index) = audio_stream_index {
                    if let Some(audio_stream_out) = audio_stream_index_out {
                        if packet.stream() == audio_index {
                            if let Some(ref mut audio_decoder) = audio_decoder {
                                audio_decoder.send_packet(&packet)?;
                                
                                while audio_decoder.receive_frame(&mut audio_decoded).is_ok() {
                                    if let Some(ref mut resampler) = resampler {
                                        resampler.run(&audio_decoded, &mut audio_encoded)?;
                                    } else {
                                        audio_encoded = audio_decoded.clone();
                                    }
                                    
                                    let out_stream = output_context.stream(audio_stream_out).unwrap();
                                    let mut out_codec = out_stream.codec();
                                    let mut encoder = out_codec.encoder().audio()?;
                                    
                                    encoder.send_frame(&audio_encoded)?;
                                    
                                    let mut out_packet = ffmpeg::packet::Packet::empty();
                                    while encoder.receive_packet(&mut out_packet).is_ok() {
                                        out_packet.set_stream(audio_stream_out);
                                        out_packet.rescale_ts(
                                            encoder.time_base(),
                                            out_stream.time_base(),
                                        );
                                        
                                        output_context.write_packet(&out_packet)?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            {
                let out_stream = output_context.stream(0).unwrap();
                let mut out_codec = out_stream.codec();
                let mut encoder = out_codec.encoder().video()?;
                
                encoder.send_eof()?;
                
                let mut out_packet = ffmpeg::packet::Packet::empty();
                while encoder.receive_packet(&mut out_packet).is_ok() {
                    out_packet.set_stream(0);
                    out_packet.rescale_ts(
                        encoder.time_base(),
                        out_stream.time_base(),
                    );
                    
                    output_context.write_packet(&out_packet)?;
                }
                
                if let Some(audio_stream_out) = audio_stream_index_out {
                    let out_stream = output_context.stream(audio_stream_out).unwrap();
                    let mut out_codec = out_stream.codec();
                    let mut encoder = out_codec.encoder().audio()?;
                    
                    encoder.send_eof()?;
                    
                    let mut out_packet = ffmpeg::packet::Packet::empty();
                    while encoder.receive_packet(&mut out_packet).is_ok() {
                        out_packet.set_stream(audio_stream_out);
                        out_packet.rescale_ts(
                            encoder.time_base(),
                            out_stream.time_base(),
                        );
                        
                        output_context.write_packet(&out_packet)?;
                    }
                }
            }
            
            output_context.write_trailer()?;
            
            {
                let mut progress_guard = progress.lock().unwrap();
                progress_guard.current_frame = total_frames;
                progress_guard.current_time = duration;
                progress_guard.percent = 100.0;
                progress_guard.complete = true;
                
                if let Some(callback) = &callback {
                    callback.lock().unwrap()(progress_guard.clone());
                }
            }
            
            Ok(())
        });
        
        self.export_thread = Some(handle);
        
        Ok(())
    }
    
    fn update_progress_with_error(
        progress: &Arc<Mutex<ExportProgress>>,
        callback: &Option<ExportCallback>,
        error_msg: &str,
    ) {
        let mut progress_guard = progress.lock().unwrap();
        progress_guard.error = Some(error_msg.to_string());
        progress_guard.complete = true;
        
        if let Some(callback) = callback {
            callback.lock().unwrap()(progress_guard.clone());
        }
    }
    
    pub fn cancel(&mut self) -> Result<(), EditingError> {
        *self.cancel_flag.lock().unwrap() = true;
        
        if let Some(handle) = self.export_thread.take() {
            if !handle.is_finished() {
                thread::sleep(Duration::from_millis(100));
                
                // If it's still not finished, we'll just let it run in the background
                // It will eventually notice the cancellation flag and terminate
            }
        }
        
        Ok(())
    }
    
   pub fn get_progress(&self) -> ExportProgress {
        self.progress.lock().unwrap().clone()
    }
    
    pub fn is_complete(&self) -> bool {
        self.progress.lock().unwrap().complete
    }
    
    pub fn has_error(&self) -> bool {
        self.progress.lock().unwrap().error.is_some()
    }
    
    pub fn get_error(&self) -> Option<String> {
        self.progress.lock().unwrap().error.clone()
    }
}
