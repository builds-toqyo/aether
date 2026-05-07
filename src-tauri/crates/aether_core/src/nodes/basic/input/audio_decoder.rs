use aether_types::{ParameterValue};
use ffmpeg as ffmpeg;
use ffmpeg::{codec, format, frame, media};
use std::ffi::CString;
use uuid::Uuid;
use log::{debug, error, warn};


pub struct AudioDecoder {

    frame_cache: std::collections::HashMap<u64, Uuid>,
}

impl AudioDecoder {

    pub fn new() -> Self {
        Self {
            frame_cache: std::collections::HashMap::new(),
        }
    }


    pub fn decode_audio_frame_with_ffmpeg(&mut self, frame: u64, media_path: &str) -> Uuid {


        debug!("Decoding audio frame {} from {}", frame, media_path);


        if let Err(e) = ffmpeg::init() {
            error!("Failed to initialize FFmpeg for audio: {}", e);
            return Uuid::new_v4();
        }


        let path_cstring = CString::new(media_path).unwrap_or_else(|_| CString::new("default.wav").unwrap());
        let mut input_format_context = match format::Input::open(&path_cstring) {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to open audio file: {}", e);
                return Uuid::new_v4();
            }
        };


        if let Err(e) = input_format_context.find_stream_info(None) {
            error!("Failed to find audio stream info: {}", e);
            return Uuid::new_v4();
        }


        let input_stream = match input_format_context.streams().best(media::Type::Audio) {
            Some(stream) => stream,
            None => {
                error!("No audio stream found in file");
                return Uuid::new_v4();
            }
        };


        let codec_params = input_stream.parameters();
        let sample_rate = codec_params.sample_rate().unwrap_or(48000);
        let channels = codec_params.channels().unwrap_or(2) as u8;
        let bit_depth = codec_params.bits_per_coded_sample().unwrap_or(16) as u8;


        let decoder = match codec::find_by_name("aac") {
            Some(decoder) => decoder,
            None => {
                error!("Audio decoder not found");
                return Uuid::new_v4();
            }
        };

        let mut decoder_context = match codec::Context::new() {
            Ok(context) => context,
            Err(e) => {
                error!("Failed to create audio decoder context: {}", e);
                return Uuid::new_v4();
            }
        };

        decoder_context.set_parameters(input_stream.parameters());

        if let Err(e) = decoder_context.open(decoder, None) {
            error!("Failed to open audio decoder: {}", e);
            return Uuid::new_v4();
        }


        let samples_per_frame = sample_rate / 30;


        let mut audio_frame = frame::Audio::new(codec::SampleFormat::F32(sample_rate), samples_per_frame, channels);


        let timestamp = frame as f64 / 30.0;
        let seek_timestamp = (timestamp * sample_rate as f64) as i64;


        let mut packet_iter = input_format_context.packets();
        let mut audio_id = Uuid::new_v4();

        if let Some((_, packet)) = packet_iter.next() {
            if let Err(e) = decoder_context.send_packet(&packet) {
                error!("Failed to send audio packet: {}", e);
                return audio_id;
            }

            if let Err(e) = decoder_context.receive_frame(&mut audio_frame) {
                error!("Failed to receive audio frame: {}", e);
                return audio_id;
            }


            let audio_data = self.extract_audio_samples(&audio_frame, channels);


            let audio_metadata = AudioMetadata {
                frame_number: frame,
                sample_rate: sample_rate as u32,
                channels,
                bit_depth,
                samples_per_frame: samples_per_frame as u32,
                audio_id,
            };

            debug!("Audio decoded via FFmpeg: {}Hz, {} channels, {} bits, {} samples/frame",
                sample_rate, channels, bit_depth, samples_per_frame);

            debug!("Audio metadata: {:?}", audio_metadata);

        } else {
            warn!("No audio packet found for frame {}", frame);
        }

        audio_id
    }


    fn extract_audio_samples(&self, frame: &frame::Audio, channels: u8) -> Vec<f32> {
        let samples = frame.samples();
        let total_samples = samples.len() * channels as usize;

        debug!("Extracting {} audio samples ({} channels)", total_samples, channels);


        let mut audio_data = Vec::with_capacity(total_samples);

        for channel_samples in samples {
            for sample in channel_samples {
                audio_data.push(*sample as f32 / i16::MAX as f32);
            }
        }

        debug!("Extracted {} audio samples", audio_data.len());

        audio_data
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
