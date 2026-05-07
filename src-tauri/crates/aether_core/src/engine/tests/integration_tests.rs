#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::engine::editing::types::*;
    use crate::engine::editing::*;
    use crate::engine::rendering::formats::*;
    use crate::engine::rendering::encoder::*;
    use crate::engine::rendering::*;
    use crate::engine::integration::*;
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use anyhow::Result;


    fn setup_test_environment() -> Result<(Arc<Mutex<EditingEngine>>, Arc<Mutex<RenderingEngine>>)> {

        init_gstreamer();


        download_test_video_if_needed()?;


        let editing_engine = create_test_editing_engine()?;
        let rendering_engine = create_test_rendering_engine()?;

        Ok((editing_engine, rendering_engine))
    }

    #[test]
    fn test_import_and_analyze_media() -> Result<()> {
        let (editing_engine, _) = setup_test_environment()?;


        let clip_id = import_test_video(&editing_engine)?;
        assert!(!clip_id.is_empty());


        let media_info = editing_engine.lock().unwrap().get_media_info(&clip_id)?;


        assert!(!media_info.uri.is_empty());
        assert!(media_info.duration > 0);
        assert!(!media_info.streams.is_empty());


        let video_streams = media_info.streams.iter()
            .filter(|s| matches!(s.stream_type, StreamType::Video))
            .collect::<Vec<_>>();

        assert!(!video_streams.is_empty());

        Ok(())
    }

    #[test]
    fn test_timeline_clip_operations() -> Result<()> {
        let (editing_engine, _) = setup_test_environment()?;


        let clip_id = import_test_video(&editing_engine)?;

        let timeline = editing_engine.lock().unwrap().timeline();
        let mut timeline = timeline.lock().unwrap();


        let video_track_id = timeline.add_track(TrackType::Video)?;


        let timeline_clip_id = timeline.add_clip_to_track(&clip_id, &video_track_id, 0)?;
        assert!(!timeline_clip_id.is_empty());


        let clip_info = timeline.get_clip_info(&timeline_clip_id)?;
        assert_eq!(clip_info.track_id, video_track_id);
        assert_eq!(clip_info.start_time, 0);


        timeline.move_clip(&timeline_clip_id, 5000000000)?;


        let updated_clip_info = timeline.get_clip_info(&timeline_clip_id)?;
        assert_eq!(updated_clip_info.start_time, 5000000000);


        timeline.trim_clip(&timeline_clip_id, 1000000000, None)?;


        let trimmed_clip_info = timeline.get_clip_info(&timeline_clip_id)?;
        assert_eq!(trimmed_clip_info.start_time, 5000000000);
        assert_eq!(trimmed_clip_info.in_point, 1000000000);


        let split_result = timeline.split_clip(&timeline_clip_id, 7000000000)?;
        assert_eq!(split_result.len(), 2);


        let clips = timeline.get_clips();
        assert_eq!(clips.len(), 2);


        timeline.remove_clip(&timeline_clip_id)?;


        let remaining_clips = timeline.get_clips();
        assert_eq!(remaining_clips.len(), 1);
        assert_ne!(remaining_clips[0].id, timeline_clip_id);

        Ok(())
    }

    #[test]
    fn test_effect_application() -> Result<()> {
        let (editing_engine, _) = setup_test_environment()?;


        let clip_id = create_simple_test_timeline(&editing_engine)?;

        let timeline = editing_engine.lock().unwrap().timeline();
        let mut timeline = timeline.lock().unwrap();


        let clips = timeline.get_clips();
        assert_eq!(clips.len(), 1);
        let timeline_clip_id = clips[0].id.clone();


        let effect_type = EffectType::VideoEffect("agingtv".to_string());
        let effect_id = timeline.add_effect_to_clip(&timeline_clip_id, effect_type.clone())?;
        assert!(!effect_id.is_empty());


        let effect_info = timeline.get_effect_info(&effect_id)?;
        assert_eq!(effect_info.clip_id, timeline_clip_id);


        timeline.set_effect_parameter(&effect_id, "scratch-lines", &"7")?;


        let params = timeline.get_effect_parameters(&effect_id)?;
        assert!(params.contains_key("scratch-lines"));


        timeline.remove_effect(&effect_id)?;


        let effects = timeline.get_effects_for_clip(&timeline_clip_id);
        assert!(effects.is_empty());

        Ok(())
    }

    #[test]
    fn test_preview_engine() -> Result<()> {
        let (editing_engine, _) = setup_test_environment()?;


        create_simple_test_timeline(&editing_engine)?;


        let mut engine = editing_engine.lock().unwrap();
        let preview = engine.create_preview(640, 360)?;


        let frame_received = Arc::new(Mutex::new(false));
        let frame_received_clone = frame_received.clone();

        preview.lock().unwrap().set_frame_callback(Box::new(move |frame| {

            assert!(!frame.data.is_empty());
            assert_eq!(frame.width, 640);
            assert_eq!(frame.height, 360);


            *frame_received_clone.lock().unwrap() = true;
        }));


        preview.lock().unwrap().play()?;


        wait_for_condition(
            || *frame_received.lock().unwrap(),
            Duration::from_secs(5),
            Duration::from_millis(100),
        )?;


        preview.lock().unwrap().stop()?;

        Ok(())
    }

    #[test]
    fn test_intermediate_export() -> Result<()> {
        let (editing_engine, _) = setup_test_environment()?;


        create_simple_test_timeline(&editing_engine)?;


        let output_path = create_test_output_path("intermediate", "mkv")?;
        let mut export_options = ExportOptions::default();
        export_options.output_path = output_path.clone();
        export_options.container = "mkv".to_string();
        export_options.video_codec = "libx264".to_string();
        export_options.audio_codec = "aac".to_string();
        export_options.video_bitrate = 2000000;
        export_options.audio_bitrate = 128000;


        let mut exporter = editing_engine.lock().unwrap()
            .create_intermediate_export(export_options)?;


        let (progress_history, callback) = create_mock_progress_callback::<ExportProgress>();
        exporter.set_progress_callback(callback);


        exporter.start_export()?;


        wait_for_condition(
            || {
                let history = progress_history.lock().unwrap();
                !history.is_empty() && history.last().unwrap().complete
            },
            Duration::from_secs(30),
            Duration::from_millis(500),
        )?;


        let history = progress_history.lock().unwrap();
        let last_progress = history.last().unwrap();
        assert!(last_progress.complete);
        assert!(last_progress.error.is_none());


        assert!(check_file_exists_with_content(&output_path));

        Ok(())
    }

    #[test]
    fn test_integrated_export() -> Result<()> {
        let (editing_engine, rendering_engine) = setup_test_environment()?;


        create_simple_test_timeline(&editing_engine)?;


        let output_path = create_test_output_path("final", "mp4")?;


        let mut export_options = ExportOptions::new(&output_path);


        export_options.gst_options.video_codec = "libx264".to_string();
        export_options.gst_options.audio_codec = "flac".to_string();


        export_options.ffmpeg_options.video_format = VideoFormat::H264;
        export_options.ffmpeg_options.audio_format = AudioFormat::Aac;
        export_options.ffmpeg_options.preset = EncoderPreset::UltraFast;


        let mut exporter = create_integrated_exporter(
            editing_engine.clone(),
            rendering_engine.clone(),
            export_options,
        )?;


        let (progress_history, callback) = create_mock_progress_callback::<ExportProgress>();
        exporter.set_progress_callback(callback);


        exporter.start_export()?;


        wait_for_condition(
            || {
                let history = progress_history.lock().unwrap();
                !history.is_empty() && history.last().unwrap().complete
            },
            Duration::from_secs(60),
            Duration::from_millis(500),
        )?;


        let history = progress_history.lock().unwrap();
        let last_progress = history.last().unwrap();
        assert!(last_progress.complete);
        assert!(last_progress.error.is_none());


        assert!(check_file_exists_with_content(&output_path));

        Ok(())
    }
}
