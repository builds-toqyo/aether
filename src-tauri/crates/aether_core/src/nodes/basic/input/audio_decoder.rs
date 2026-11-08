use ffmpeg::{format, frame, media};
use ffmpeg_next as ffmpeg;
use log::{debug, error, warn};
use uuid::Uuid;

pub struct AudioDecoder {
    _frame_cache: std::collections::HashMap<u64, Uuid>,
}

impl AudioDecoder {
    pub fn new() -> Self {
        Self {
            _frame_cache: std::collections::HashMap::new(),
        }
    }

    pub fn decode_audio_frame_with_ffmpeg(&mut self, frame: u64, media_path: &str) -> Uuid {
        debug!("Decoding audio frame {} from {}", frame, media_path);

        if let Err(e) = ffmpeg::init() {
            error!("Failed to initialize FFmpeg for audio: {}", e);
            return Uuid::new_v4();
        }

        let mut input_format_context = match format::input(media_path) {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to open audio file: {}", e);
                return Uuid::new_v4();
            }
        };

        let input_stream = match input_format_context.streams().best(media::Type::Audio) {
            Some(stream) => stream,
            None => {
                error!("No audio stream found in file");
                return Uuid::new_v4();
            }
        };

        let mut decoder =
            match ffmpeg::codec::context::Context::from_parameters(input_stream.parameters()) {
                Ok(context) => match context.decoder().audio() {
                    Ok(decoder) => decoder,
                    Err(e) => {
                        error!("Failed to create audio decoder: {}", e);
                        return Uuid::new_v4();
                    }
                },
                Err(e) => {
                    error!("Failed to create decoder context: {}", e);
                    return Uuid::new_v4();
                }
            };

        let sample_rate = decoder.rate() as u32;
        let channels = decoder.channels() as u8;
        let format = decoder.format();
        let samples_per_frame = sample_rate / 30;
        let bit_depth: u8 = match format.name() {
            "s16" | "s16p" | "s16be" | "s16le" => 16,
            "s32" | "s32p" | "s32be" | "s32le" => 32,
            "flt" | "fltp" => 32,
            "dbl" | "dblp" => 64,
            "u8" | "u8p" => 8,
            _ => 16,
        };

        let mut audio_frame = frame::Audio::empty();

        let mut packet_iter = input_format_context.packets();
        let audio_id = Uuid::new_v4();

        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder.send_packet(&packet) {
                error!("Failed to send audio packet: {}", e);
                return audio_id;
            }

            if let Err(e) = decoder.receive_frame(&mut audio_frame) {
                error!("Failed to receive audio frame: {}", e);
                return audio_id;
            }

            let _audio_data = self.extract_audio_samples(&audio_frame, channels);

            let audio_metadata = AudioMetadata {
                frame_number: frame,
                sample_rate,
                channels,
                bit_depth,
                samples_per_frame,
                audio_id,
            };

            debug!(
                "Audio decoded via FFmpeg: {}Hz, {} channels, {} bits, {} samples/frame",
                sample_rate, channels, bit_depth, samples_per_frame
            );

            debug!("Audio metadata: {:?}", audio_metadata);
        } else {
            warn!("No audio packet found for frame {}", frame);
        }

        audio_id
    }

    fn extract_audio_samples(&self, frame: &frame::Audio, channels: u8) -> Vec<f32> {
        use ffmpeg::format::sample::Sample;

        fn decode_plane(data: &[u8], sample_fmt: Sample) -> Vec<f32> {
            match sample_fmt {
                Sample::U8(_) => data.iter().map(|&v| (v as f32 - 128.0) / 128.0).collect(),
                Sample::I16(_) => data
                    .chunks_exact(2)
                    .map(|c| i16::from_ne_bytes([c[0], c[1]]) as f32 / i16::MAX as f32)
                    .collect(),
                Sample::I32(_) => data
                    .chunks_exact(4)
                    .map(|c| i32::from_ne_bytes([c[0], c[1], c[2], c[3]]) as f32 / i32::MAX as f32)
                    .collect(),
                Sample::I64(_) => data
                    .chunks_exact(8)
                    .map(|c| i64::from_ne_bytes(c.try_into().unwrap()) as f32 / i64::MAX as f32)
                    .collect(),
                Sample::F32(_) => data
                    .chunks_exact(4)
                    .map(|c| f32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
                    .collect(),
                Sample::F64(_) => data
                    .chunks_exact(8)
                    .map(|c| f64::from_ne_bytes(c.try_into().unwrap()) as f32)
                    .collect(),
                Sample::None => Vec::new(),
            }
        }

        let sample_fmt = frame.format();
        let channels = (channels as usize).max(1);
        let sample_count = frame.samples();

        if frame.is_packed() {
            // Interleaved samples live in plane 0
            let mut samples = decode_plane(frame.data(0), sample_fmt);
            samples.truncate(sample_count * channels);
            samples
        } else {
            // Planar layout: one plane per channel, deinterleave into output
            let plane_count = frame.planes().min(channels);
            let channel_data: Vec<Vec<f32>> = (0..plane_count)
                .map(|ch| decode_plane(frame.data(ch), sample_fmt))
                .collect();

            let mut samples = Vec::with_capacity(sample_count * channels);
            for i in 0..sample_count {
                for ch in 0..channels {
                    let sample = channel_data
                        .get(ch)
                        .and_then(|c| c.get(i))
                        .copied()
                        .unwrap_or(0.0);
                    samples.push(sample);
                }
            }
            samples
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioMetadata {
    pub frame_number: u64,
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub samples_per_frame: u32,
    pub audio_id: Uuid,
}

impl Default for AudioDecoder {
    fn default() -> Self {
        Self::new()
    }
}
