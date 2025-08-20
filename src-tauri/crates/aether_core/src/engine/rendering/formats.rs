use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContainerFormat {
    Mp4,
    Mkv,
    Mov,
    Webm,
    Avi,
    Flv,
    Wmv,
    Mpg,
    Ts,
    Mxf,
    Gif,
}

impl ContainerFormat {
    pub fn to_ffmpeg_name(&self) -> &'static str {
        match self {
            ContainerFormat::Mp4 => "mp4",
            ContainerFormat::Mkv => "matroska",
            ContainerFormat::Mov => "mov",
            ContainerFormat::Webm => "webm",
            ContainerFormat::Avi => "avi",
            ContainerFormat::Flv => "flv",
            ContainerFormat::Wmv => "asf",
            ContainerFormat::Mpg => "mpegts",
            ContainerFormat::Ts => "mpegts",
            ContainerFormat::Mxf => "mxf",
            ContainerFormat::Gif => "gif",
        }
    }
    
    pub fn extension(&self) -> &'static str {
        match self {
            ContainerFormat::Mp4 => "mp4",
            ContainerFormat::Mkv => "mkv",
            ContainerFormat::Mov => "mov",
            ContainerFormat::Webm => "webm",
            ContainerFormat::Avi => "avi",
            ContainerFormat::Flv => "flv",
            ContainerFormat::Wmv => "wmv",
            ContainerFormat::Mpg => "mpg",
            ContainerFormat::Ts => "ts",
            ContainerFormat::Mxf => "mxf",
            ContainerFormat::Gif => "gif",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            ContainerFormat::Mp4 => "MP4",
            ContainerFormat::Mkv => "Matroska (MKV)",
            ContainerFormat::Mov => "QuickTime (MOV)",
            ContainerFormat::Webm => "WebM",
            ContainerFormat::Avi => "AVI",
            ContainerFormat::Flv => "Flash Video (FLV)",
            ContainerFormat::Wmv => "Windows Media (WMV)",
            ContainerFormat::Mpg => "MPEG",
            ContainerFormat::Ts => "MPEG Transport Stream (TS)",
            ContainerFormat::Mxf => "Material Exchange Format (MXF)",
            ContainerFormat::Gif => "GIF Animation",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VideoFormat {
    H264,
    H265,
    Vp8,
    Vp9,
    Av1,
    ProRes,
    Dnxhd,
    Mjpeg,
    Mpeg2,
    Mpeg4,
    Theora,
    Raw,
}

impl VideoFormat {
    pub fn to_ffmpeg_name(&self) -> &'static str {
        match self {
            VideoFormat::H264 => "libx264",
            VideoFormat::H265 => "libx265",
            VideoFormat::Vp8 => "libvpx",
            VideoFormat::Vp9 => "libvpx-vp9",
            VideoFormat::Av1 => "libaom-av1",
            VideoFormat::ProRes => "prores_ks",
            VideoFormat::Dnxhd => "dnxhd",
            VideoFormat::Mjpeg => "mjpeg",
            VideoFormat::Mpeg2 => "mpeg2video",
            VideoFormat::Mpeg4 => "mpeg4",
            VideoFormat::Theora => "libtheora",
            VideoFormat::Raw => "rawvideo",
        }
    }
    
=    pub fn display_name(&self) -> &'static str {
        match self {
            VideoFormat::H264 => "H.264 / AVC",
            VideoFormat::H265 => "H.265 / HEVC",
            VideoFormat::Vp8 => "VP8",
            VideoFormat::Vp9 => "VP9",
            VideoFormat::Av1 => "AV1",
            VideoFormat::ProRes => "Apple ProRes",
            VideoFormat::Dnxhd => "Avid DNxHD",
            VideoFormat::Mjpeg => "Motion JPEG",
            VideoFormat::Mpeg2 => "MPEG-2",
            VideoFormat::Mpeg4 => "MPEG-4",
            VideoFormat::Theora => "Theora",
            VideoFormat::Raw => "Uncompressed",
        }
    }
    
    pub fn is_compatible_with(&self, container: ContainerFormat) -> bool {
        match container {
            ContainerFormat::Mp4 => matches!(
                self,
                VideoFormat::H264 | VideoFormat::H265 | VideoFormat::Mpeg4
            ),
            ContainerFormat::Mkv => true, // MKV supports all codecs
            ContainerFormat::Mov => matches!(
                self,
                VideoFormat::H264 | VideoFormat::H265 | VideoFormat::ProRes | VideoFormat::Mjpeg
            ),
            ContainerFormat::Webm => matches!(
                self,
                VideoFormat::Vp8 | VideoFormat::Vp9
            ),
            ContainerFormat::Avi => matches!(
                self,
                VideoFormat::Mjpeg | VideoFormat::Mpeg4
            ),
            ContainerFormat::Flv => matches!(
                self,
                VideoFormat::H264
            ),
            ContainerFormat::Wmv => matches!(
                self,
                VideoFormat::Mpeg4
            ),
            ContainerFormat::Mpg | ContainerFormat::Ts => matches!(
                self,
                VideoFormat::Mpeg2 | VideoFormat::H264
            ),
            ContainerFormat::Mxf => matches!(
                self,
                VideoFormat::Dnxhd | VideoFormat::Mpeg2
            ),
            ContainerFormat::Gif => matches!(
                self,
                VideoFormat::Mjpeg
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AudioFormat {
    Aac,
    Mp3,
    Opus,
    Vorbis,
    Flac,
    Pcm,
    Ac3,
    Eac3,
    Wma,
}

impl AudioFormat {
=    pub fn to_ffmpeg_name(&self) -> &'static str {
        match self {
            AudioFormat::Aac => "aac",
            AudioFormat::Mp3 => "libmp3lame",
            AudioFormat::Opus => "libopus",
            AudioFormat::Vorbis => "libvorbis",
            AudioFormat::Flac => "flac",
            AudioFormat::Pcm => "pcm_s16le",
            AudioFormat::Ac3 => "ac3",
            AudioFormat::Eac3 => "eac3",
            AudioFormat::Wma => "wmav2",
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            AudioFormat::Aac => "AAC",
            AudioFormat::Mp3 => "MP3",
            AudioFormat::Opus => "Opus",
            AudioFormat::Vorbis => "Vorbis",
            AudioFormat::Flac => "FLAC",
            AudioFormat::Pcm => "PCM (Uncompressed)",
            AudioFormat::Ac3 => "Dolby Digital (AC-3)",
            AudioFormat::Eac3 => "Dolby Digital Plus (E-AC-3)",
            AudioFormat::Wma => "Windows Media Audio",
        }
    }
    
    pub fn is_compatible_with(&self, container: ContainerFormat) -> bool {
        match container {
            ContainerFormat::Mp4 => matches!(
                self,
                AudioFormat::Aac | AudioFormat::Ac3 | AudioFormat::Eac3
            ),
            ContainerFormat::Mkv => true, // MKV supports all codecs
            ContainerFormat::Mov => matches!(
                self,
                AudioFormat::Aac | AudioFormat::Pcm
            ),
            ContainerFormat::Webm => matches!(
                self,
                AudioFormat::Opus | AudioFormat::Vorbis
            ),
            ContainerFormat::Avi => matches!(
                self,
                AudioFormat::Mp3 | AudioFormat::Pcm
            ),
            ContainerFormat::Flv => matches!(
                self,
                AudioFormat::Aac | AudioFormat::Mp3
            ),
            ContainerFormat::Wmv => matches!(
                self,
                AudioFormat::Wma
            ),
            ContainerFormat::Mpg | ContainerFormat::Ts => matches!(
                self,
                AudioFormat::Mp3 | AudioFormat::Ac3
            ),
            ContainerFormat::Mxf => matches!(
                self,
                AudioFormat::Pcm
            ),
            ContainerFormat::Gif => false, // GIF has no audio
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInfo {
    pub container: ContainerFormat,
    
    pub video_formats: Vec<VideoFormat>,
    
    pub audio_formats: Vec<AudioFormat>,
    
    pub use_case: String,
    
    pub web_friendly: bool,
}

pub fn get_available_formats() -> Vec<FormatInfo> {
    let containers = vec![
        ContainerFormat::Mp4,
        ContainerFormat::Mkv,
        ContainerFormat::Mov,
        ContainerFormat::Webm,
        ContainerFormat::Avi,
        ContainerFormat::Flv,
        ContainerFormat::Wmv,
        ContainerFormat::Mpg,
        ContainerFormat::Ts,
        ContainerFormat::Mxf,
        ContainerFormat::Gif,
    ];
    
    let video_formats = vec![
        VideoFormat::H264,
        VideoFormat::H265,
        VideoFormat::Vp8,
        VideoFormat::Vp9,
        VideoFormat::Av1,
        VideoFormat::ProRes,
        VideoFormat::Dnxhd,
        VideoFormat::Mjpeg,
        VideoFormat::Mpeg2,
        VideoFormat::Mpeg4,
        VideoFormat::Theora,
        VideoFormat::Raw,
    ];
    
    let audio_formats = vec![
        AudioFormat::Aac,
        AudioFormat::Mp3,
        AudioFormat::Opus,
        AudioFormat::Vorbis,
        AudioFormat::Flac,
        AudioFormat::Pcm,
        AudioFormat::Ac3,
        AudioFormat::Eac3,
        AudioFormat::Wma,
    ];
    
    let use_cases = HashMap::from([
        (ContainerFormat::Mp4, "Web, mobile, and general purpose"),
        (ContainerFormat::Mkv, "High quality archival and storage"),
        (ContainerFormat::Mov, "Professional video editing and Apple devices"),
        (ContainerFormat::Webm, "Web video optimized for browsers"),
        (ContainerFormat::Avi, "Legacy format with wide compatibility"),
        (ContainerFormat::Flv, "Legacy web streaming format"),
        (ContainerFormat::Wmv, "Windows-specific playback"),
        (ContainerFormat::Mpg, "DVD and broadcast compatibility"),
        (ContainerFormat::Ts, "Broadcast and streaming"),
        (ContainerFormat::Mxf, "Professional broadcast and archival"),
        (ContainerFormat::Gif, "Short animations without audio"),
    ]);
    
    let web_friendly = vec![
        ContainerFormat::Mp4,
        ContainerFormat::Webm,
        ContainerFormat::Gif,
    ];
    
    containers.into_iter().map(|container| {
        let compatible_video = video_formats.iter()
            .filter(|format| format.is_compatible_with(container))
            .copied()
            .collect();
        
        let compatible_audio = audio_formats.iter()
            .filter(|format| format.is_compatible_with(container))
            .copied()
            .collect();
        
        FormatInfo {
            container,
            video_formats: compatible_video,
            audio_formats: compatible_audio,
            use_case: use_cases.get(&container).unwrap_or(&"General purpose").to_string(),
            web_friendly: web_friendly.contains(&container),
        }
    }).collect()
}
