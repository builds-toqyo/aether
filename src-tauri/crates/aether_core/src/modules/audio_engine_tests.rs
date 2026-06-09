use super::audio_engine::*;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[test]
fn test_audio_engine_creation() -> Result<()> {

    let engine = AudioEngine::new()?;


    assert_eq!(engine.master_volume(), 1.0);
    assert!(!engine.initialized);
    assert!(engine.tracks.is_empty());

    Ok(())
}

#[test]
fn test_audio_engine_initialization() -> Result<()> {

    let mut engine = AudioEngine::new()?;


    engine.initialize()?;


    assert!(engine.initialized);
    assert!(engine.pipeline.is_some());
    assert!(engine.mixer.is_some());
    assert!(engine.master_volume_element.is_some());
    assert!(engine.bus_watch_id.is_some());


    assert!(!engine.devices.is_empty());


    engine.shutdown()?;


    assert!(!engine.initialized);
    assert!(engine.bus_watch_id.is_none());

    Ok(())
}

#[test]
fn test_audio_track_creation() -> Result<()> {

    let track = AudioTrack::new("test-track", AudioSourceType::File { path: "test.mp3".to_string() });


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

    let mut track = AudioTrack::new("test-track", AudioSourceType::File { path: "test.mp3".to_string() });


    track.initialize()?;


    track.set_volume(0.5)?;
    assert_eq!(track.volume_level, 0.5);


    track.set_pan(-0.3)?;
    assert_eq!(track.pan_position, -0.3);


    track.set_mute(true)?;
    assert!(track.muted);


    track.set_solo(true)?;
    assert!(track.soloed);

    Ok(())
}

#[test]
fn test_audio_engine_track_management() -> Result<()> {

    let mut engine = AudioEngine::new()?;


    engine.add_track("track1", AudioSourceType::File { path: "test.mp3".to_string() })?;


    assert_eq!(engine.tracks.len(), 1);
    assert!(engine.get_track("track1").is_some());


    let tracks = engine.get_tracks();
    assert_eq!(tracks.len(), 1);


    engine.remove_track("track1")?;


    assert!(engine.tracks.is_empty());
    assert!(engine.get_track("track1").is_none());

    Ok(())
}

#[test]
fn test_audio_engine_master_volume() -> Result<()> {

    let mut engine = AudioEngine::new()?;


    engine.initialize()?;


    engine.set_master_volume(0.7)?;


    assert_eq!(engine.master_volume(), 0.7);

    Ok(())
}

#[test]
fn test_audio_device_management() -> Result<()> {

    let mut engine = AudioEngine::new()?;


    engine.initialize()?;


    assert!(!engine.devices.is_empty());


    let output_devices = engine.get_output_devices();
    assert!(!output_devices.is_empty());


    let default_output = engine.get_default_output_device();
    assert!(default_output.is_some());


    if let Some(device) = default_output {
        let found_device = engine.get_device_by_id(&device.id);
        assert!(found_device.is_some());
    }

    Ok(())
}

#[test]
fn test_audio_effects() -> Result<()> {

    let mut track = AudioTrack::new("test-track", AudioSourceType::File { path: "test.mp3".to_string() });


    track.initialize()?;


    let bands = vec![100.0, 1000.0, 10000.0];
    let gains = vec![0.0, 3.0, -3.0];
    track.add_effect(AudioEffectType::Equalizer { bands, gains })?;


    assert_eq!(track.effects.len(), 1);


    track.add_effect(AudioEffectType::Reverb {
        room_size: 0.8,
        damping: 0.5,
        wet_level: 0.3,
        dry_level: 0.7
    })?;


    assert_eq!(track.effects.len(), 2);


    track.remove_effect(0)?;


    assert_eq!(track.effects.len(), 1);


    track.clear_effects()?;


    assert!(track.effects.is_empty());

    Ok(())
}


#[test]
#[ignore]
fn test_audio_playback() -> Result<()> {

    let mut engine = AudioEngine::new()?;


    engine.add_track("track1", AudioSourceType::File { path: "test.mp3".to_string() })?;


    engine.play()?;


    thread::sleep(Duration::from_secs(2));


    engine.pause()?;


    thread::sleep(Duration::from_secs(1));


    engine.play()?;


    thread::sleep(Duration::from_secs(2));


    engine.stop()?;


    engine.shutdown()?;

    Ok(())
}
