use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
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
            video_bitrate: 2_000_000,
            audio_bitrate: 128_000,
            frame_rate: 30.0,
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
    cancel_flag: Arc<Mutex<bool>>,
}

impl Exporter {
    pub fn new(options: ExportOptions) -> Result<Self, EditingError> {
        ffmpeg::init().map_err(|e| EditingError::ExportError(format!("FFmpeg initialization failed: {}", e)))?;

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

        thread::spawn(move || {
            let input_path = options.input_path.to_string_lossy().to_string();
            let mut input_context = match ffmpeg::format::input(&input_path) {
                Ok(ctx) => ctx,
                Err(e) => {
                    let error_msg = format!("Failed to open input file: {}", e);
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    return Err(EditingError::ExportError(error_msg));
                }
            };

            // ffmpeg-next 8.x removed Input::dump
            // ffmpeg::format::context::input::Input::dump(&input_context, 0, None);

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
                let decoder = codec_context.decoder().video()?;

                let width = decoder.width();
                let height = decoder.height();

                let frame_rate = {
                    let rate = stream.avg_frame_rate();
                    rate.numerator() as f64 / rate.denominator() as f64
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

            let _format_name = options.container_format.to_ffmpeg_name();

            let video_codec_name = options.video_format.to_ffmpeg_name();
            let video_codec = ffmpeg::encoder::find_by_name(video_codec_name)
                .ok_or_else(|| {
                    let error_msg = format!("Video codec not found: {}", video_codec_name);
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    EditingError::ExportError(error_msg)
                })?;

            let mut video_stream = output_context.add_stream(video_codec)?;

            {
                let context = ffmpeg::codec::context::Context::new_with_codec(video_codec);
                let mut video = context.encoder().video()?;

                let out_width = if options.width > 0 { options.width } else { width as u32 };
                let out_height = if options.height > 0 { options.height } else { height as u32 };
                video.set_width(out_width);
                video.set_height(out_height);

                video.set_format(ffmpeg::format::pixel::Pixel::YUV420P);

                let out_frame_rate = if options.frame_rate > 0.0 { options.frame_rate } else { frame_rate };
                let frame_rate_rational = ffmpeg::util::rational::Rational::new(
                    (out_frame_rate * 1000.0) as i32,
                    1000,
                );
                video.set_time_base(frame_rate_rational.invert());
                video_stream.set_time_base(frame_rate_rational.invert());

                if options.video_bitrate > 0 {
                    video.set_bit_rate(options.video_bitrate as usize);
                }

                let mut dict = ffmpeg::Dictionary::new();
                if options.video_bitrate <= 0 {
                    dict.set("crf", &options.crf.to_string());
                }
                dict.set("preset", options.encoder_preset.to_ffmpeg_name());
                if options.threads > 0 {
                    dict.set("threads", &options.threads.to_string());
                }

                let opened = video.open_with(dict)?;
                video_stream.set_parameters(opened);
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
                    let _input_codec_par = input_stream.parameters();

                    let context = ffmpeg::codec::context::Context::new_with_codec(audio_codec);
                    let mut audio = context.encoder().audio()?;

                    audio.set_rate(48000);
                    audio.set_channel_layout(ffmpeg::channel_layout::ChannelLayout::STEREO);
                    audio.set_format(ffmpeg::format::sample::Sample::F32(ffmpeg::format::sample::Type::Planar));

                    let time_base = ffmpeg::util::rational::Rational::new(1, 48000);
                    audio.set_time_base(time_base);
                    audio_stream.set_time_base(time_base);

                    if options.audio_bitrate > 0 {
                        audio.set_bit_rate(options.audio_bitrate as usize);
                    }

                    let opened = audio.open()?;
                    audio_stream.set_parameters(opened);
                }
            }

            output_context.write_header()?;

            let mut video_decoder = {
                let stream = input_context.stream(video_stream_index.unwrap()).unwrap();
                let context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
                context.decoder().video()?
            };
            let video_time_base = input_context.stream(video_stream_index.unwrap()).unwrap().time_base();

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
                let out_codec_context = ffmpeg::codec::context::Context::from_parameters(out_stream.parameters())?;
                let out_codec = out_codec_context.encoder().audio()?;

                Some(ffmpeg::software::resampling::context::Context::get(
                    audio_decoder.format(),
                    audio_decoder.channel_layout(),
                    audio_decoder.rate(),
                    ffmpeg::format::sample::Sample::F32(ffmpeg::format::sample::Type::Planar),
                    out_codec.channel_layout(),
                    out_codec.rate(),
                )?)
            } else {
                None
            };

            let mut decoded = ffmpeg::frame::Video::new(
                video_decoder.format(),
                video_decoder.width(),
                video_decoder.height(),
            );

            let mut encoded = ffmpeg::frame::Video::new(
                ffmpeg::format::pixel::Pixel::YUV420P,
                if options.width > 0 { options.width } else { width as u32 },
                if options.height > 0 { options.height } else { height as u32 },
            );

            let mut audio_decoded = ffmpeg::frame::Audio::empty();
            let mut audio_encoded = ffmpeg::frame::Audio::empty();
            let _packet = ffmpeg::packet::Packet::empty();

            let mut frame_count = 0;

            for (stream, packet) in input_context.packets() {
                if *cancel_flag.lock().unwrap() {
                    let error_msg = "Export cancelled".to_string();
                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                    return Err(EditingError::ExportError(error_msg));
                }

                if let Some(stream_index) = video_stream_index {
                    if stream.index() == stream_index {
                        video_decoder.send_packet(&packet)?;

                        while video_decoder.receive_frame(&mut decoded).is_ok() {

                            encoded = ffmpeg::frame::Video::new(
                                ffmpeg::format::pixel::Pixel::YUV420P,
                                if options.width > 0 { options.width } else { width as u32 },
                                if options.height > 0 { options.height } else { height as u32 },
                            );

                            scaler.run(&decoded, &mut encoded)?;

                            let pts = packet.pts().unwrap_or(0);
                            let pts_seconds = pts as f64 * f64::from(video_time_base);

                            encoded.set_pts(Some(frame_count as i64));

                            let out_stream = output_context.stream(0).unwrap();
                            let out_time_base = out_stream.time_base();
                            let out_codec_context = ffmpeg::codec::context::Context::from_parameters(out_stream.parameters())?;
                            let mut encoder = out_codec_context.encoder().video()?;

                            encoder.send_frame(&encoded)?;

                            let mut out_packet = ffmpeg::packet::Packet::empty();
                            while encoder.receive_packet(&mut out_packet).is_ok() {
                                out_packet.set_stream(0);
                                out_packet.rescale_ts(
                                    encoder.time_base(),
                                    out_time_base,
                                );

                                out_packet.write_interleaved(&mut output_context)?;
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
                        if stream.index() == audio_index {
                            if let Some(ref mut audio_decoder) = audio_decoder {

                                if *cancel_flag.lock().unwrap() {
                                    let error_msg = "Export cancelled during audio processing".to_string();
                                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                                    return Err(EditingError::ExportError(error_msg));
                                }


                                if let Err(e) = audio_decoder.send_packet(&packet) {
                                    let error_msg = format!("Audio decoder error: {}", e);
                                    Self::update_progress_with_error(&progress, &callback, &error_msg);
                                    return Err(EditingError::ExportError(error_msg));
                                }


                                let mut audio_frame_result = audio_decoder.receive_frame(&mut audio_decoded);

                                while audio_frame_result.is_ok() {

                                    audio_encoded = ffmpeg::frame::Audio::empty();


                                    if let Some(ref mut resampler) = resampler {
                                        if let Err(e) = resampler.run(&audio_decoded, &mut audio_encoded) {
                                            let error_msg = format!("Audio resampling error: {}", e);
                                            Self::update_progress_with_error(&progress, &callback, &error_msg);
                                            return Err(EditingError::ExportError(error_msg));
                                        }
                                    } else {
                                        audio_encoded = audio_decoded.clone();
                                    }

                                    let out_stream = output_context.stream(audio_stream_out).unwrap();
                                    let out_time_base = out_stream.time_base();
                                    let out_codec_context = ffmpeg::codec::context::Context::from_parameters(out_stream.parameters())?;
                                    let mut encoder = match out_codec_context.encoder().audio() {
                                        Ok(enc) => enc,
                                        Err(e) => {
                                            let error_msg = format!("Audio encoder error: {}", e);
                                            Self::update_progress_with_error(&progress, &callback, &error_msg);
                                            return Err(EditingError::ExportError(error_msg));
                                        }
                                    };

                                    if let Err(e) = encoder.send_frame(&audio_encoded) {
                                        let error_msg = format!("Audio encoding error: {}", e);
                                        Self::update_progress_with_error(&progress, &callback, &error_msg);
                                        return Err(EditingError::ExportError(error_msg));
                                    }

                                    let mut out_packet = ffmpeg::packet::Packet::empty();
                                    let mut packet_result = encoder.receive_packet(&mut out_packet);

                                    while packet_result.is_ok() {
                                        out_packet.set_stream(audio_stream_out);
                                        out_packet.rescale_ts(
                                            encoder.time_base(),
                                            out_time_base,
                                        );

                                        if let Err(e) = out_packet.write_interleaved(&mut output_context) {
                                            let error_msg = format!("Error writing audio packet: {}", e);
                                            Self::update_progress_with_error(&progress, &callback, &error_msg);
                                            return Err(EditingError::ExportError(error_msg));
                                        }

                                        packet_result = encoder.receive_packet(&mut out_packet);
                                    }

                                    audio_frame_result = audio_decoder.receive_frame(&mut audio_decoded);
                                }
                            }
                        }
                    }
                }
            }

            {
                {
                    let out_stream = output_context.stream(0).unwrap();
                    let out_stream_time_base = out_stream.time_base();
                    let out_codec_context = ffmpeg::codec::context::Context::from_parameters(out_stream.parameters())?;
                    let mut encoder = out_codec_context.encoder().video()?;

                    encoder.send_eof()?;

                    let mut out_packet = ffmpeg::packet::Packet::empty();
                    while encoder.receive_packet(&mut out_packet).is_ok() {
                        out_packet.set_stream(0);
                        out_packet.rescale_ts(
                            encoder.time_base(),
                            out_stream_time_base,
                        );

                        out_packet.write_interleaved(&mut output_context)?;
                    }
                }

                if let Some(audio_stream_out) = audio_stream_index_out {
                    let out_stream = output_context.stream(audio_stream_out).unwrap();
                    let out_stream_time_base = out_stream.time_base();
                    let out_codec_context = ffmpeg::codec::context::Context::from_parameters(out_stream.parameters())?;
                    let mut encoder = out_codec_context.encoder().audio()?;

                    encoder.send_eof()?;

                    let mut out_packet = ffmpeg::packet::Packet::empty();
                    while encoder.receive_packet(&mut out_packet).is_ok() {
                        out_packet.set_stream(audio_stream_out);
                        out_packet.rescale_ts(
                            encoder.time_base(),
                            out_stream_time_base,
                        );

                        out_packet.write_interleaved(&mut output_context)?;
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
