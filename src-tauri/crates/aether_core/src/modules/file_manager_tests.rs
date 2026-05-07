#[cfg(test)]
mod tests {
    use super::super::file_manager::{FileManager, MediaType, ThumbnailOptions};
    use std::path::Path;
    use std::fs;
    use std::io::Write;
    use anyhow::Result;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;


    fn create_test_file(name: &str, content: &[u8]) -> Result<std::path::PathBuf> {
        let temp_dir = std::env::temp_dir().join("aether_test");
        fs::create_dir_all(&temp_dir)?;

        let file_path = temp_dir.join(name);
        let mut file = fs::File::create(&file_path)?;
        file.write_all(content)?;

        Ok(file_path)
    }

    #[test]
    fn test_determine_media_type() -> Result<()> {
        let file_manager = FileManager::new()?;


        let video_path = create_test_file("test.mp4", b"dummy video data")?;
        let audio_path = create_test_file("test.mp3", b"dummy audio data")?;
        let image_path = create_test_file("test.jpg", b"dummy image data")?;
        let unknown_path = create_test_file("test.xyz", b"unknown data")?;


        let video_info = file_manager.get_media_info(&video_path)?;
        assert_eq!(video_info.media_type, MediaType::Video);

        let audio_info = file_manager.get_media_info(&audio_path)?;
        assert_eq!(audio_info.media_type, MediaType::Audio);

        let image_info = file_manager.get_media_info(&image_path)?;
        assert_eq!(image_info.media_type, MediaType::Image);

        let unknown_info = file_manager.get_media_info(&unknown_path)?;
        assert_eq!(unknown_info.media_type, MediaType::Unknown);


        fs::remove_file(video_path)?;
        fs::remove_file(audio_path)?;
        fs::remove_file(image_path)?;
        fs::remove_file(unknown_path)?;

        Ok(())
    }

    #[test]
    fn test_copy_file_with_progress() -> Result<()> {
        let file_manager = FileManager::new()?;


        let source_path = create_test_file("source.dat", &[0u8; 1024 * 1024])?;
        let dest_path = std::env::temp_dir().join("aether_test").join("dest.dat");


        let bytes_copied = Arc::new(AtomicU64::new(0));
        let bytes_copied_clone = bytes_copied.clone();


        file_manager.copy_file(&source_path, &dest_path, move |copied, total| {
            bytes_copied_clone.store(copied, Ordering::SeqCst);
            assert!(copied <= total);
        })?;


        assert_eq!(bytes_copied.load(Ordering::SeqCst), 1024 * 1024);
        assert!(dest_path.exists());
        assert_eq!(fs::metadata(&dest_path)?.len(), 1024 * 1024);


        fs::remove_file(source_path)?;
        fs::remove_file(dest_path)?;

        Ok(())
    }

    #[test]
    fn test_media_info_cache() -> Result<()> {
        let file_manager = FileManager::new()?;


        let file_path = create_test_file("test_cache.mp4", b"dummy video data")?;


        let info1 = file_manager.get_media_info(&file_path)?;
        let info2 = file_manager.get_media_info(&file_path)?;


        assert_eq!(info1.path, info2.path);
        assert_eq!(info1.media_type, info2.media_type);
        assert_eq!(info1.size, info2.size);


        fs::remove_file(file_path)?;

        Ok(())
    }


    #[test]
    #[ignore]
    fn test_thumbnail_generation() -> Result<()> {
        let file_manager = FileManager::new()?;


        let image_path = create_test_file("test_thumb.jpg", b"dummy image data")?;


        let options = ThumbnailOptions {
            width: 120,
            height: 80,
            position: None,
            quality: 85,
        };


        let result = file_manager.generate_thumbnail(&image_path, Some(options));


        fs::remove_file(image_path)?;


        Ok(())
    }

    #[test]
    fn test_cleanup() -> Result<()> {
        let file_manager = FileManager::new()?;


        let file_path = create_test_file("test_cleanup.mp4", b"dummy video data")?;
        let _info = file_manager.get_media_info(&file_path)?;


        file_manager.cleanup()?;


        fs::remove_file(file_path)?;

        Ok(())
    }
}
