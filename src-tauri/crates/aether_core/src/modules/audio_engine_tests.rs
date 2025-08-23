use super::audio_engine::*;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_audio_engine_creation() -> Result<()> {
    // Create a new audio engine with default configuration
    let engine = AudioEngine::new()?;
    
    // Check that the engine was created with default values
    assert_eq!(engine.master_volume(), 1.0);
    assert!(!engine.initialized);
    assert!(engine.tracks.is_empty());
    
    Ok(())
}

#[test]
fn test_audio_engine_initialization() -> Result<()> {
    // Create a new audio engine
    let mut engine = AudioEngine::new()?;
    
    // Initialize the engine
    engine.initialize()?;
    
    // Check that the engine was initialized
    assert!(engine.initialized);
    assert!(engine.pipeline.is_some());
    assert!(engine.mixer.is_some());
    assert!(engine.master_volume_element.is_some());
    assert!(engine.bus_watch_id.is_some());
    
    // Check that devices were loaded
    assert!(!engine.devices.is_empty());
    
    // Shutdown the engine
    engine.shutdown()?;
    
    // Check that the engine was shut down
    assert!(!engine.initialized);
    assert!(engine.bus_watch_id.is_none());
    
    Ok(())
}

#[test]
fn test_audio_track_creation() -> Result<()> {
    // Create a new audio track
    let track = AudioTrack::new("test-track", AudioSourceType::File { path: "test.mp3".to_string() });
    
    // Check that the track was created with default values
    assert_eq!(track.id, "test-track");
    assert_eq!(track.volume_level, 1.0);
    assert_eq!(track.pan_position, 0.0);
    assert!(!track.muted);
    assert!(!track.soloed);
    assert!(matches!(track.playback_state, PlaybackState::Stopped));
    assert!(track.effects.is_empty());
    
    Ok(())
}

#[test]
fn test_audio_track_volume_pan() -> Result<()> {
    // Create a new audio track
    let mut track = AudioTrack::new("test-track", AudioSourceType::File { path: "test.mp3".to_string() });
    
    // Initialize the track
    track.initialize()?;
    
    // Set volume
    track.set_volume(0.5)?;
    assert_eq!(track.volume_level, 0.5);
    
    // Set pan
    track.set_pan(-0.3)?;
    assert_eq!(track.pan_position, -0.3);
    
    // Set mute
    track.set_mute(true)?;
    assert!(track.muted);
    
    // Set solo
    track.set_solo(true)?;
    assert!(track.soloed);
    
    Ok(())
}

#[test]
fn test_audio_engine_track_management() -> Result<()> {
    // Create a new audio engine
    let mut engine = AudioEngine::new()?;
    
    // Add a track
    engine.add_track("track1", AudioSourceType::File { path: "test.mp3".to_string() })?;
    
    // Check that the track was added
    assert_eq!(engine.tracks.len(), 1);
    assert!(engine.get_track("track1").is_some());
    
    // Get all tracks
    let tracks = engine.get_tracks();
    assert_eq!(tracks.len(), 1);
    
    // Remove the track
    engine.remove_track("track1")?;
    
    // Check that the track was removed
    assert!(engine.tracks.is_empty());
    assert!(engine.get_track("track1").is_none());
    
    Ok(())
}

#[test]
fn test_audio_engine_master_volume() -> Result<()> {
    // Create a new audio engine
    let mut engine = AudioEngine::new()?;
    
    // Initialize the engine
    engine.initialize()?;
    
    // Set master volume
    engine.set_master_volume(0.7)?;
    
    // Check that the master volume was set
    assert_eq!(engine.master_volume(), 0.7);
    
    Ok(())
}

#[test]
fn test_audio_device_management() -> Result<()> {
    // Create a new audio engine
    let mut engine = AudioEngine::new()?;
    
    // Initialize the engine
    engine.initialize()?;
    
    // Check that devices were loaded
    assert!(!engine.devices.is_empty());
    
    // Get output devices
    let output_devices = engine.get_output_devices();
    assert!(!output_devices.is_empty());
    
    // Get default output device
    let default_output = engine.get_default_output_device();
    assert!(default_output.is_some());
    
    // Get device by ID
    if let Some(device) = default_output {
        let found_device = engine.get_device_by_id(&device.id);
        assert!(found_device.is_some());
    }
    
    Ok(())
}

#[test]
fn test_audio_effects() -> Result<()> {
    // Create a new audio track
    let mut track = AudioTrack::new("test-track", AudioSourceType::File { path: "test.mp3".to_string() });
    
    // Initialize the track
    track.initialize()?;
    
    // Add an equalizer effect
    let bands = vec![100.0, 1000.0, 10000.0];
    let gains = vec![0.0, 3.0, -3.0];
    track.add_effect(AudioEffectType::Equalizer { bands, gains })?;
    
    // Check that the effect was added
    assert_eq!(track.effects.len(), 1);
    
    // Add a reverb effect
    track.add_effect(AudioEffectType::Reverb { 
        room_size: 0.8, 
        damping: 0.5, 
        wet_level: 0.3, 
        dry_level: 0.7 
    })?;
    
    // Check that the effect was added
    assert_eq!(track.effects.len(), 2);
    
    // Remove the first effect
    track.remove_effect(0)?;
    
    // Check that the effect was removed
    assert_eq!(track.effects.len(), 1);
    
    // Clear all effects
    track.clear_effects()?;
    
    // Check that all effects were removed
    assert!(track.effects.is_empty());
    
    Ok(())
}

// This test is marked as ignore because it actually plays audio
// which is not suitable for automated testing
#[test]
#[ignore]
fn test_audio_playback() -> Result<()> {
    // Create a new audio engine
    let mut engine = AudioEngine::new()?;
    
    // Add a track with a test audio file
    // Note: Replace with an actual audio file path for manual testing
    engine.add_track("track1", AudioSourceType::File { path: "test.mp3".to_string() })?;
    
    // Play the audio
    engine.play()?;
    
    // Wait for a moment
    thread::sleep(Duration::from_secs(2));
    
    // Pause the audio
    engine.pause()?;
    
    // Wait for a moment
    thread::sleep(Duration::from_secs(1));
    
    // Resume playback
    engine.play()?;
    
    // Wait for a moment
    thread::sleep(Duration::from_secs(2));
    
    // Stop the audio
    engine.stop()?;
    
    // Shutdown the engine
    engine.shutdown()?;
    
    Ok(())
}
