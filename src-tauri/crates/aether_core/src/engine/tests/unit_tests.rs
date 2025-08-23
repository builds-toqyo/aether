#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::engine::editing::types::*;
    use crate::engine::editing::*;
    use crate::engine::rendering::formats::*;
    use crate::engine::rendering::encoder::*;
    use crate::engine::rendering::*;
    use std::path::PathBuf;
    use std::time::Duration;
    
    #[test]
    fn test_editing_engine_initialization() {
        let result = create_test_editing_engine();
        assert!(result.is_ok(), "Failed to create editing engine: {:?}", result.err());
        
        let engine = result.unwrap();
        let project_name = engine.lock().unwrap().project_name();
        assert_eq!(project_name, "test_project");
    }
    
    #[test]
    fn test_timeline_track_management() {
        let engine = create_test_editing_engine().unwrap();
        let timeline = engine.lock().unwrap().timeline();
        let mut timeline = timeline.lock().unwrap();
        
        // Add video track
        let video_track_id = timeline.add_track(TrackType::Video).unwrap();
        assert!(!video_track_id.is_empty());
        
        // Add audio track
        let audio_track_id = timeline.add_track(TrackType::Audio).unwrap();
        assert!(!audio_track_id.is_empty());
        
        // Get tracks
        let tracks = timeline.get_tracks();
        assert_eq!(tracks.len(), 2);
        
        // Remove track
        let result = timeline.remove_track(&video_track_id);
        assert!(result.is_ok());
        
        // Verify track was removed
        let tracks = timeline.get_tracks();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].id, audio_track_id);
    }
    
    #[test]
    fn test_container_format_properties() {
        let mp4 = ContainerFormat::Mp4;
        assert_eq!(mp4.to_ffmpeg_name(), "mp4");
        assert_eq!(mp4.extension(), "mp4");
        assert_eq!(mp4.display_name(), "MP4");
        
        let mkv = ContainerFormat::Mkv;
        assert_eq!(mkv.to_ffmpeg_name(), "matroska");
        assert_eq!(mkv.extension(), "mkv");
        assert_eq!(mkv.display_name(), "Matroska (MKV)");
    }
    
    #[test]
    fn test_video_format_properties() {
        let h264 = VideoFormat::H264;
        assert_eq!(h264.to_ffmpeg_name(), "libx264");
        assert_eq!(h264.display_name(), "H.264 / AVC");
        assert!(h264.is_compatible_with(ContainerFormat::Mp4));
        assert!(h264.is_compatible_with(ContainerFormat::Mkv));
        assert!(!h264.is_compatible_with(ContainerFormat::Webm));
        
        let vp9 = VideoFormat::Vp9;
        assert_eq!(vp9.to_ffmpeg_name(), "libvpx-vp9");
        assert_eq!(vp9.display_name(), "VP9");
        assert!(vp9.is_compatible_with(ContainerFormat::Webm));
        assert!(vp9.is_compatible_with(ContainerFormat::Mkv));
        assert!(!vp9.is_compatible_with(ContainerFormat::Mp4));
    }
    
    #[test]
    fn test_audio_format_properties() {
        let aac = AudioFormat::Aac;
        assert_eq!(aac.to_ffmpeg_name(), "aac");
        assert_eq!(aac.display_name(), "AAC");
        assert!(aac.is_compatible_with(ContainerFormat::Mp4));
        assert!(aac.is_compatible_with(ContainerFormat::Mkv));
        assert!(!aac.is_compatible_with(ContainerFormat::Webm));
        
        let opus = AudioFormat::Opus;
        assert_eq!(opus.to_ffmpeg_name(), "libopus");
        assert_eq!(opus.display_name(), "Opus");
        assert!(opus.is_compatible_with(ContainerFormat::Webm));
        assert!(opus.is_compatible_with(ContainerFormat::Mkv));
        assert!(!opus.is_compatible_with(ContainerFormat::Mp4));
    }
    
    #[test]
    fn test_encoder_options() {
        let default_options = EncoderOptions::default();
        assert_eq!(default_options.video_format, VideoFormat::H264);
        assert_eq!(default_options.audio_format, AudioFormat::Aac);
        assert_eq!(default_options.preset, EncoderPreset::Medium);
        
        let high_quality = EncoderOptions::high_quality();
        assert_eq!(high_quality.video_format, VideoFormat::H265);
        assert_eq!(high_quality.audio_format, AudioFormat::Flac);
        assert_eq!(high_quality.preset, EncoderPreset::Slow);
        assert_eq!(high_quality.crf, 18);
        assert!(high_quality.two_pass);
        
        let web_delivery = EncoderOptions::web_delivery();
        assert_eq!(web_delivery.video_format, VideoFormat::H264);
        assert_eq!(web_delivery.audio_format, AudioFormat::Aac);
        assert_eq!(web_delivery.preset, EncoderPreset::Medium);
        assert_eq!(web_delivery.crf, 23);
        assert!(!web_delivery.two_pass);
        
        let mut custom = EncoderOptions::default();
        custom.with_preset(EncoderPreset::Fast)
              .with_crf(20)
              .with_video_bitrate(5000000)
              .with_audio_bitrate(192000)
              .with_two_pass(true)
              .add_option("profile:v", "high");
        
        assert_eq!(custom.preset, EncoderPreset::Fast);
        assert_eq!(custom.crf, 20);
        assert_eq!(custom.video_bitrate, 5000000);
        assert_eq!(custom.audio_bitrate, 192000);
        assert!(custom.two_pass);
        assert_eq!(custom.additional_options.get("profile:v").unwrap(), "high");
    }
    
    #[test]
    fn test_encoder_preset_properties() {
        let medium = EncoderPreset::Medium;
        assert_eq!(medium.to_ffmpeg_name(), "medium");
        assert!(medium.description().contains("Balanced"));
        
        let veryslow = EncoderPreset::VerySlow;
        assert_eq!(veryslow.to_ffmpeg_name(), "veryslow");
        assert!(veryslow.description().contains("Extremely slow"));
    }
    
    #[test]
    fn test_format_compatibility() {
        let formats = get_available_formats();
        assert!(!formats.is_empty());
        
        // Check MP4 format
        let mp4_format = formats.iter().find(|f| f.container == ContainerFormat::Mp4).unwrap();
        assert!(mp4_format.video_formats.contains(&VideoFormat::H264));
        assert!(mp4_format.audio_formats.contains(&AudioFormat::Aac));
        assert!(mp4_format.web_friendly);
        
        // Check MKV format
        let mkv_format = formats.iter().find(|f| f.container == ContainerFormat::Mkv).unwrap();
        assert!(mkv_format.video_formats.contains(&VideoFormat::H264));
        assert!(mkv_format.video_formats.contains(&VideoFormat::H265));
        assert!(mkv_format.audio_formats.contains(&AudioFormat::Flac));
        assert!(!mkv_format.web_friendly);
        
        // Check WebM format
        let webm_format = formats.iter().find(|f| f.container == ContainerFormat::Webm).unwrap();
        assert!(webm_format.video_formats.contains(&VideoFormat::Vp9));
        assert!(webm_format.audio_formats.contains(&AudioFormat::Opus));
        assert!(webm_format.web_friendly);
    }
    
    #[test]
    fn test_encoder_ffmpeg_args() {
        let options = EncoderOptions::web_delivery();
        let args = options.to_ffmpeg_args();
        
        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx264".to_string()));
        assert!(args.contains(&"-c:a".to_string()));
        assert!(args.contains(&"aac".to_string()));
        assert!(args.contains(&"-preset".to_string()));
        assert!(args.contains(&"medium".to_string()));
        assert!(args.contains(&"-crf".to_string()));
        assert!(args.contains(&"23".to_string()));
        
        let mut custom = EncoderOptions::default();
        custom.video_format = VideoFormat::H265;
        custom.with_video_bitrate(5000000);
        
        let args = custom.to_ffmpeg_args();
        assert!(args.contains(&"-c:v".to_string()));
        assert!(args.contains(&"libx265".to_string()));
        assert!(args.contains(&"-b:v".to_string()));
        assert!(args.contains(&"5000k".to_string()));
    }
    
    #[test]
    fn test_editing_error_conversion() {
        let error = EditingError::InvalidOperation("Test error".to_string());
        let anyhow_error = anyhow::Error::from(error.clone());
        assert!(anyhow_error.to_string().contains("Test error"));
        
        let display_str = format!("{}", error);
        assert!(display_str.contains("Test error"));
    }
}
