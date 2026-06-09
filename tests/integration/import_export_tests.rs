#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::collections::HashMap;


    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ImportFormat {

        Mp4,
        Mov,
        Avi,
        Mkv,
        Webm,

        Mp3,
        Wav,
        Aac,
        Flac,

        Png,
        Jpg,
        Tiff,
        Exr,
    }

    impl ImportFormat {
        pub fn from_extension(ext: &str) -> Option<Self> {
            match ext.to_lowercase().as_str() {
                "mp4" => Some(ImportFormat::Mp4),
                "mov" => Some(ImportFormat::Mov),
                "avi" => Some(ImportFormat::Avi),
                "mkv" => Some(ImportFormat::Mkv),
                "webm" => Some(ImportFormat::Webm),
                "mp3" => Some(ImportFormat::Mp3),
                "wav" => Some(ImportFormat::Wav),
                "aac" => Some(ImportFormat::Aac),
                "flac" => Some(ImportFormat::Flac),
                "png" => Some(ImportFormat::Png),
                "jpg" | "jpeg" => Some(ImportFormat::Jpg),
                "tiff" | "tif" => Some(ImportFormat::Tiff),
                "exr" => Some(ImportFormat::Exr),
                _ => None,
            }
        }

        pub fn is_video(&self) -> bool {
            matches!(self, ImportFormat::Mp4 | ImportFormat::Mov | ImportFormat::Avi | ImportFormat::Mkv | ImportFormat::Webm)
        }

        pub fn is_audio(&self) -> bool {
            matches!(self, ImportFormat::Mp3 | ImportFormat::Wav | ImportFormat::Aac | ImportFormat::Flac)
        }

        pub fn is_image(&self) -> bool {
            matches!(self, ImportFormat::Png | ImportFormat::Jpg | ImportFormat::Tiff | ImportFormat::Exr)
        }
    }


    #[derive(Debug, Clone)]
    pub struct ExportSettings {
        pub format: ExportFormat,
        pub resolution: (u32, u32),
        pub frame_rate: f64,
        pub video_codec: VideoCodec,
        pub video_bitrate: u32,
        pub audio_codec: AudioCodec,
        pub audio_bitrate: u32,
        pub quality_preset: QualityPreset,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum ExportFormat {
        Mp4,
        Mov,
        Webm,
        Gif,
        PngSequence,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum VideoCodec {
        H264,
        H265,
        Vp9,
        ProRes,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum AudioCodec {
        Aac,
        Mp3,
        Opus,
        Pcm,
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum QualityPreset {
        Draft,
        Preview,
        Standard,
        High,
        Master,
    }

    impl Default for ExportSettings {
        fn default() -> Self {
            Self {
                format: ExportFormat::Mp4,
                resolution: (1920, 1080),
                frame_rate: 30.0,
                video_codec: VideoCodec::H264,
                video_bitrate: 10_000_000,
                audio_codec: AudioCodec::Aac,
                audio_bitrate: 192_000,
                quality_preset: QualityPreset::Standard,
            }
        }
    }


    #[derive(Debug, Clone)]
    pub struct ImportResult {
        pub success: bool,
        pub media_id: Option<String>,
        pub format: Option<ImportFormat>,
        pub duration_ms: Option<u64>,
        pub resolution: Option<(u32, u32)>,
        pub error: Option<String>,
    }


    #[derive(Debug, Clone)]
    pub struct ExportProgress {
        pub current_frame: u64,
        pub total_frames: u64,
        pub percent: f64,
        pub elapsed_ms: u64,
        pub estimated_remaining_ms: u64,
        pub status: ExportStatus,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum ExportStatus {
        Preparing,
        Encoding,
        Finalizing,
        Complete,
        Failed(String),
        Cancelled,
    }


    pub struct MockImporter {
        supported_formats: Vec<ImportFormat>,
        max_file_size: u64,
    }

    impl MockImporter {
        pub fn new() -> Self {
            Self {
                supported_formats: vec![
                    ImportFormat::Mp4, ImportFormat::Mov, ImportFormat::Avi,
                    ImportFormat::Mkv, ImportFormat::Webm,
                    ImportFormat::Mp3, ImportFormat::Wav, ImportFormat::Aac, ImportFormat::Flac,
                    ImportFormat::Png, ImportFormat::Jpg, ImportFormat::Tiff, ImportFormat::Exr,
                ],
                max_file_size: 10 * 1024 * 1024 * 1024,
            }
        }

        pub fn import(&self, path: &str) -> ImportResult {
            let path_buf = PathBuf::from(path);


            let ext = path_buf.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            let format = match ImportFormat::from_extension(ext) {
                Some(f) => f,
                None => return ImportResult {
                    success: false,
                    media_id: None,
                    format: None,
                    duration_ms: None,
                    resolution: None,
                    error: Some(format!("Unsupported format: {}", ext)),
                },
            };

            if !self.supported_formats.contains(&format) {
                return ImportResult {
                    success: false,
                    media_id: None,
                    format: Some(format),
                    duration_ms: None,
                    resolution: None,
                    error: Some("Format not supported".to_string()),
                };
            }


            let media_id = format!("media_{:08x}", rand_mock());
            let (duration, resolution) = if format.is_video() {
                (Some(10000), Some((1920, 1080)))
            } else if format.is_audio() {
                (Some(180000), None)
            } else {
                (None, Some((1920, 1080)))
            };

            ImportResult {
                success: true,
                media_id: Some(media_id),
                format: Some(format),
                duration_ms: duration,
                resolution,
                error: None,
            }
        }

        pub fn validate_file(&self, path: &str) -> Result<(), String> {
            let path_buf = PathBuf::from(path);

            let ext = path_buf.extension()
                .and_then(|e| e.to_str())
                .ok_or("No file extension")?;

            ImportFormat::from_extension(ext)
                .ok_or_else(|| format!("Unsupported format: {}", ext))?;

            Ok(())
        }

        pub fn get_supported_extensions(&self) -> Vec<&'static str> {
            vec![__STRING_21__, __STRING_22__, __STRING_23__, __STRING_24__, __STRING_25__, __STRING_26__, __STRING_27__, __STRING_28__, __STRING_29__, __STRING_30__, __STRING_31__, __STRING_32__, __STRING_33__, __STRING_34__, __STRING_35__]
        }
    }

    /// Mock exporter for testing
    pub struct MockExporter {
        settings: ExportSettings,
        progress: ExportProgress,
    }

    impl MockExporter {
        pub fn new(settings: ExportSettings) -> Self {
            Self {
                settings,
                progress: ExportProgress {
                    current_frame: 0,
                    total_frames: 0,
                    percent: 0.0,
                    elapsed_ms: 0,
                    estimated_remaining_ms: 0,
                    status: ExportStatus::Preparing,
                },
            }
        }

        pub fn validate_settings(&self) -> Result<(), Vec<String>> {
            let mut errors = Vec::new();

            if self.settings.resolution.0 == 0 || self.settings.resolution.1 == 0 {
                errors.push(__STRING_36__.to_string());
            }

            if self.settings.frame_rate <= 0.0 {
                errors.push(__STRING_37__.to_string());
            }

            if self.settings.video_bitrate == 0 {
                errors.push(__STRING_38__.to_string());
            }

            // Check codec compatibility
            match self.settings.format {
                ExportFormat::Webm => {
                    if self.settings.video_codec != VideoCodec::Vp9 {
                        errors.push(__STRING_39__.to_string());
                    }
                    if self.settings.audio_codec != AudioCodec::Opus {
                        errors.push(__STRING_40__.to_string());
                    }
                }
                ExportFormat::Gif => {
                    if self.settings.audio_codec != AudioCodec::Pcm {
                        // GIF has no audio, but we'll allow any setting
                    }
                }
                _ => {}
            }

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }

        pub fn start_export(&mut self, duration_ms: u64) -> Result<(), String> {
            self.validate_settings().map_err(|e| e.join(", "))?;

            let total_frames = (duration_ms as f64 / 1000.0 * self.settings.frame_rate) as u64;
            self.progress = ExportProgress {
                current_frame: 0,
                total_frames,
                percent: 0.0,
                elapsed_ms: 0,
                estimated_remaining_ms: 0,
                status: ExportStatus::Encoding,
            };

            Ok(())
        }

        pub fn update_progress(&mut self, current_frame: u64, elapsed_ms: u64) {
            self.progress.current_frame = current_frame;
            self.progress.elapsed_ms = elapsed_ms;
            self.progress.percent = (current_frame as f64 / self.progress.total_frames as f64) * 100.0;

            if current_frame > 0 {
                let ms_per_frame = elapsed_ms as f64 / current_frame as f64;
                let remaining_frames = self.progress.total_frames - current_frame;
                self.progress.estimated_remaining_ms = (remaining_frames as f64 * ms_per_frame) as u64;
            }
        }

        pub fn complete(&mut self) {
            self.progress.status = ExportStatus::Complete;
            self.progress.percent = 100.0;
        }

        pub fn cancel(&mut self) {
            self.progress.status = ExportStatus::Cancelled;
        }

        pub fn get_progress(&self) -> &ExportProgress {
            &self.progress
        }

        pub fn estimate_file_size(&self, duration_ms: u64) -> u64 {
            let video_size = (self.settings.video_bitrate as u64 * duration_ms) / 8000;
            let audio_size = (self.settings.audio_bitrate as u64 * duration_ms) / 8000;
            video_size + audio_size
        }
    }

    fn rand_mock() -> u32 {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(12345);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }

    #[test]
    fn test_import_video() {
        let importer = MockImporter::new();
        let result = importer.import("/path/to/video.mp4");

        assert!(result.success);
        assert!(result.media_id.is_some());
        assert_eq!(result.format, Some(ImportFormat::Mp4));
        assert!(result.duration_ms.is_some());
        assert!(result.resolution.is_some());
    }

    #[test]
    fn test_import_audio() {
        let importer = MockImporter::new();
        let result = importer.import("/path/to/audio.mp3");

        assert!(result.success);
        assert_eq!(result.format, Some(ImportFormat::Mp3));
        assert!(result.duration_ms.is_some());
        assert!(result.resolution.is_none());
    }

    #[test]
    fn test_import_image() {
        let importer = MockImporter::new();
        let result = importer.import("/path/to/image.png");

        assert!(result.success);
        assert_eq!(result.format, Some(ImportFormat::Png));
        assert!(result.duration_ms.is_none());
        assert!(result.resolution.is_some());
    }

    #[test]
    fn test_import_unsupported_format() {
        let importer = MockImporter::new();
        let result = importer.import("/path/to/file.xyz");

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("Unsupported"));
    }

    #[test]
    fn test_format_detection() {
        assert_eq!(ImportFormat::from_extension("mp4"), Some(ImportFormat::Mp4));
        assert_eq!(ImportFormat::from_extension("MP4"), Some(ImportFormat::Mp4));
        assert_eq!(ImportFormat::from_extension("jpg"), Some(ImportFormat::Jpg));
        assert_eq!(ImportFormat::from_extension("jpeg"), Some(ImportFormat::Jpg));
        assert_eq!(ImportFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_format_categories() {
        assert!(ImportFormat::Mp4.is_video());
        assert!(!ImportFormat::Mp4.is_audio());
        assert!(!ImportFormat::Mp4.is_image());

        assert!(ImportFormat::Mp3.is_audio());
        assert!(!ImportFormat::Mp3.is_video());

        assert!(ImportFormat::Png.is_image());
        assert!(!ImportFormat::Png.is_video());
    }

    #[test]
    fn test_export_settings_validation() {
        let settings = ExportSettings::default();
        let exporter = MockExporter::new(settings);

        assert!(exporter.validate_settings().is_ok());
    }

    #[test]
    fn test_export_invalid_resolution() {
        let mut settings = ExportSettings::default();
        settings.resolution = (0, 0);
        let exporter = MockExporter::new(settings);

        let result = exporter.validate_settings();
        assert!(result.is_err());
    }

    #[test]
    fn test_export_webm_codec_requirement() {
        let mut settings = ExportSettings::default();
        settings.format = ExportFormat::Webm;
        settings.video_codec = VideoCodec::H264;
        let exporter = MockExporter::new(settings);

        let result = exporter.validate_settings();
        assert!(result.is_err());
    }

    #[test]
    fn test_export_progress() {
        let settings = ExportSettings::default();
        let mut exporter = MockExporter::new(settings);

        exporter.start_export(10000).unwrap();

        let progress = exporter.get_progress();
        assert_eq!(progress.status, ExportStatus::Encoding);
        assert_eq!(progress.total_frames, 300);
    }

    #[test]
    fn test_export_progress_update() {
        let settings = ExportSettings::default();
        let mut exporter = MockExporter::new(settings);

        exporter.start_export(10000).unwrap();
        exporter.update_progress(150, 5000);

        let progress = exporter.get_progress();
        assert_eq!(progress.current_frame, 150);
        assert!((progress.percent - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_export_completion() {
        let settings = ExportSettings::default();
        let mut exporter = MockExporter::new(settings);

        exporter.start_export(10000).unwrap();
        exporter.complete();

        let progress = exporter.get_progress();
        assert_eq!(progress.status, ExportStatus::Complete);
        assert_eq!(progress.percent, 100.0);
    }

    #[test]
    fn test_export_cancellation() {
        let settings = ExportSettings::default();
        let mut exporter = MockExporter::new(settings);

        exporter.start_export(10000).unwrap();
        exporter.cancel();

        let progress = exporter.get_progress();
        assert_eq!(progress.status, ExportStatus::Cancelled);
    }

    #[test]
    fn test_file_size_estimation() {
        let settings = ExportSettings::default();
        let exporter = MockExporter::new(settings);


        let size = exporter.estimate_file_size(60000);


        assert!(size > 70_000_000);
        assert!(size < 80_000_000);
    }

    #[test]
    fn test_batch_import() {
        let importer = MockImporter::new();
        let files = vec![
            "/video1.mp4",
            "/video2.mov",
            "/audio.mp3",
            "/image.png",
        ];

        let results: Vec<_> = files.iter()
            .map(|f| importer.import(f))
            .collect();

        assert!(results.iter().all(|r| r.success));
        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_supported_extensions() {
        let importer = MockImporter::new();
        let extensions = importer.get_supported_extensions();

        assert!(extensions.contains(&"mp4"));
        assert!(extensions.contains(&"mov"));
        assert!(extensions.contains(&"mp3"));
        assert!(extensions.contains(&"png"));
    }
}
