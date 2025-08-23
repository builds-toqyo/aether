use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::{debug, info, warn, error};
use gst::prelude::*;
use glib;

use crate::engine::editing::types::EditingError;

/// Audio playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    /// Audio is stopped
    Stopped,
    /// Audio is playing
    Playing,
    /// Audio is paused
    Paused,
}

/// Audio source type
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioSourceType {
    /// File-based audio source
    File(PathBuf),
    /// URI-based audio source
    Uri(String),
    /// Raw audio data
    Raw(Vec<u8>, String), // (data, mime_type)
}

/// Audio effect type
#[derive(Debug, Clone)]
pub enum AudioEffectType {
    /// Equalizer effect
    Equalizer {
        /// Band frequencies in Hz
        bands: Vec<f64>,
        /// Band gains in dB
        gains: Vec<f64>,
    },
    /// Reverb effect
    Reverb {
        /// Room size (0.0 - 1.0)
        room_size: f64,
        /// Damping factor (0.0 - 1.0)
        damping: f64,
        /// Wet level (0.0 - 1.0)
        wet_level: f64,
        /// Dry level (0.0 - 1.0)
        dry_level: f64,
    },
    /// Delay effect
    Delay {
        /// Delay time in milliseconds
        time_ms: u64,
        /// Feedback amount (0.0 - 1.0)
        feedback: f64,
        /// Wet/dry mix (0.0 - 1.0)
        mix: f64,
    },
    /// Compressor effect
    Compressor {
        /// Threshold in dB
        threshold: f64,
        /// Ratio (1.0 - 20.0)
        ratio: f64,
        /// Attack time in milliseconds
        attack: f64,
        /// Release time in milliseconds
        release: f64,
        /// Makeup gain in dB
        makeup: f64,
    },
}

/// Audio track representing a single audio source with effects
pub struct AudioTrack {
    /// Track ID
    id: String,
    /// Audio source
    source: AudioSourceType,
    /// GStreamer pipeline
    pipeline: Option<gst::Pipeline>,
    /// Audio bin element
    audio_bin: Option<gst::Bin>,
    /// Volume element
    volume: Option<gst::Element>,
    /// Pan element
    pan: Option<gst::Element>,
    /// Level meter element
    level: Option<gst::Element>,
    /// Volume level (0.0 - 1.0)
    volume_level: f64,
    /// Pan position (-1.0 left to 1.0 right)
    pan_position: f64,
    /// Whether the track is muted
    muted: bool,
    /// Whether the track is soloed
    soloed: bool,
    /// Current playback state
    playback_state: PlaybackState,
    /// List of effects
    effects: Vec<gst::Element>,
    /// Peak level values (RMS) for left and right channels
    peak_levels: (f64, f64),
    /// Signal watch ID for level meter
    level_watch_id: Option<glib::SourceId>,
}

impl AudioTrack {
    /// Create a new audio track with the given ID and source
    pub fn new(id: &str, source: AudioSourceType) -> Self {
        Self {
            id: id.to_string(),
            source,
            pipeline: None,
            audio_bin: None,
            volume: None,
            pan: None,
            level: None,
            volume_level: 1.0,
            pan_position: 0.0,
            muted: false,
            soloed: false,
            playback_state: PlaybackState::Stopped,
            effects: Vec::new(),
            peak_levels: (0.0, 0.0),
            level_watch_id: None,
        }
    }
    
    /// Initialize the track's GStreamer pipeline
    pub fn initialize(&mut self) -> Result<(), EditingError> {
        // Create a new pipeline
        let pipeline = gst::Pipeline::new(Some(&format!("audio-track-{}", self.id)));
        
        // Create a bin for the audio processing chain
        let audio_bin = gst::Bin::new(Some(&format!("audio-bin-{}", self.id)));
        
        // Create source element based on the audio source type
        let source_element = match &self.source {
            AudioSourceType::File(path) => {
                // Convert path to URI
                let uri = gst::filename_to_uri(path.to_str().unwrap())
                    .map_err(|_| EditingError::AudioError(format!("Failed to convert path to URI: {:?}", path)))?;
                
                // Create filesrc element
                let filesrc = gst::ElementFactory::make("filesrc")
                    .name(&format!("source-{}", self.id))
                    .property("location", path.to_str().unwrap())
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create filesrc element".to_string()))?;
                
                // Create decodebin element
                let decodebin = gst::ElementFactory::make("decodebin")
                    .name(&format!("decode-{}", self.id))
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create decodebin element".to_string()))?;
                
                // Add elements to the bin
                audio_bin.add_many(&[&filesrc, &decodebin])
                    .map_err(|_| EditingError::AudioError("Failed to add elements to bin".to_string()))?;
                
                // Link filesrc to decodebin
                filesrc.link(&decodebin)
                    .map_err(|_| EditingError::AudioError("Failed to link filesrc to decodebin".to_string()))?;
                
                // Connect pad-added signal to handle dynamic pads
                let bin_weak = audio_bin.downgrade();
                decodebin.connect_pad_added(move |_, src_pad| {
                    if let Some(bin) = bin_weak.upgrade() {
                        handle_pad_added(&bin, src_pad);
                    }
                });
                
                filesrc
            },
            AudioSourceType::Uri(uri) => {
                // Create uridecodebin element
                let uridecodebin = gst::ElementFactory::make("uridecodebin")
                    .name(&format!("source-{}", self.id))
                    .property("uri", uri.as_str())
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create uridecodebin element".to_string()))?;
                
                // Add element to the bin
                audio_bin.add(&uridecodebin)
                    .map_err(|_| EditingError::AudioError("Failed to add uridecodebin to bin".to_string()))?;
                
                // Connect pad-added signal to handle dynamic pads
                let bin_weak = audio_bin.downgrade();
                uridecodebin.connect_pad_added(move |_, src_pad| {
                    if let Some(bin) = bin_weak.upgrade() {
                        handle_pad_added(&bin, src_pad);
                    }
                });
                
                uridecodebin
            },
            AudioSourceType::Raw(data, mime_type) => {
                // Create appsrc element
                let appsrc = gst::ElementFactory::make("appsrc")
                    .name(&format!("source-{}", self.id))
                    .property("format", gst::Format::Time)
                    .property("is-live", false)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create appsrc element".to_string()))?;
                
                // Set caps based on mime type
                let caps = gst::Caps::from_string(mime_type)
                    .map_err(|_| EditingError::AudioError(format!("Invalid mime type: {}", mime_type)))?;
                appsrc.set_property("caps", &caps);
                
                // Create decodebin element
                let decodebin = gst::ElementFactory::make("decodebin")
                    .name(&format!("decode-{}", self.id))
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create decodebin element".to_string()))?;
                
                // Add elements to the bin
                audio_bin.add_many(&[&appsrc, &decodebin])
                    .map_err(|_| EditingError::AudioError("Failed to add elements to bin".to_string()))?;
                
                // Link appsrc to decodebin
                appsrc.link(&decodebin)
                    .map_err(|_| EditingError::AudioError("Failed to link appsrc to decodebin".to_string()))?;
                
                // Connect pad-added signal to handle dynamic pads
                let bin_weak = audio_bin.downgrade();
                decodebin.connect_pad_added(move |_, src_pad| {
                    if let Some(bin) = bin_weak.upgrade() {
                        handle_pad_added(&bin, src_pad);
                    }
                });
                
                // Push data to appsrc
                let buffer = gst::Buffer::from_slice(data.clone());
                let appsrc = appsrc.dynamic_cast::<gst_app::AppSrc>().unwrap();
                appsrc.push_buffer(buffer).unwrap();
                appsrc.end_of_stream().unwrap();
                
                appsrc.upcast()
            },
        };
        
        // Create the audio bin
        let audio_bin = gst::Bin::new(Some(&format!("audio-bin-{}", self.id)));
        
        // Create the volume element
        let volume = gst::ElementFactory::make("volume")
            .name(&format!("volume-{}", self.id))
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create volume element".to_string()))?;
        
        // Create the pan element
        let pan = gst::ElementFactory::make("audiopanorama")
            .name(&format!("pan-{}", self.id))
            .property("method", 1) // Use psychoacoustic panning
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create pan element".to_string()))?;
        
        // Create the level meter element
        let level = gst::ElementFactory::make("level")
            .name(&format!("level-{}", self.id))
            .property("interval", 100_000_000u64) // 100ms in nanoseconds
            .property("peak-ttl", 500_000_000u64) // 500ms in nanoseconds
            .property("peak-falloff", 20.0) // dB per second
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create level meter element".to_string()))?;
        
        // Create the audioconvert element
        let convert = gst::ElementFactory::make("audioconvert")
            .name(&format!("convert-{}", self.id))
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create audioconvert element".to_string()))?;
        
        // Create the audioresample element
        let resample = gst::ElementFactory::make("audioresample")
            .name(&format!("resample-{}", self.id))
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create audioresample element".to_string()))?;
        
        // Add elements to the bin
        audio_bin.add_many(&[&volume, &pan, &level, &convert, &resample])
            .map_err(|_| EditingError::AudioError("Failed to add elements to bin".to_string()))?;
        
        // Link the elements
        gst::Element::link_many(&[&volume, &pan, &level, &convert, &resample])
            .map_err(|_| EditingError::AudioError("Failed to link elements".to_string()))?;
        
        // Add ghost pad to the bin
        let src_pad = resample.static_pad("src").unwrap();
        let ghost_pad = gst::GhostPad::with_target(Some("src"), &src_pad).unwrap();
        audio_bin.add_pad(&ghost_pad).unwrap();
        
        // Add the bin to the pipeline
        pipeline.add(&audio_bin)
            .map_err(|_| EditingError::AudioError("Failed to add bin to pipeline".to_string()))?;
        
        // Create a fake sink for standalone playback
        let sink = gst::ElementFactory::make("autoaudiosink")
            .name(&format!("sink-{}", self.id))
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create sink element".to_string()))?;
        
        // Add sink to the pipeline
        pipeline.add(&sink)
            .map_err(|_| EditingError::AudioError("Failed to add sink to pipeline".to_string()))?;
        
        // Link the bin to the sink
        audio_bin.link(&sink)
            .map_err(|_| EditingError::AudioError("Failed to link bin to sink".to_string()))?;
        
        // Store the elements
        self.pipeline = Some(pipeline);
        self.audio_bin = Some(audio_bin);
        self.volume = Some(volume);
        self.pan = Some(pan);
        self.level = Some(level);
        
        // Set up level meter signal watch
        let track_id = self.id.clone();
        let level_weak = level.downgrade();
        let level_watch_id = level.connect("message::element", false, move |_, msg| {
            if let Some(level) = level_weak.upgrade() {
                if msg.src().as_ref() == Some(level.upcast_ref::<gst::Object>()) {
                    if let gst::MessageView::Element(element_msg) = msg.view() {
                        let structure = element_msg.structure().unwrap();
                        if structure.name() == "level" {
                            // Get the peak RMS values
                            if let Ok(rms_values) = structure.get::<glib::ValueArray>("rms") {
                                let mut peak_levels = (0.0, 0.0);
                                
                                // Get the first channel (left)
                                if let Some(value) = rms_values.get(0) {
                                    if let Ok(level_db) = value.get::<f64>() {
                                        // Convert from dB to linear (0.0 - 1.0)
                                        let linear = if level_db > -90.0 {
                                            10.0f64.powf(level_db / 20.0)
                                        } else {
                                            0.0
                                        };
                                        peak_levels.0 = linear;
                                    }
                                }
                                
                                // Get the second channel (right) if available
                                if let Some(value) = rms_values.get(1) {
                                    if let Ok(level_db) = value.get::<f64>() {
                                        // Convert from dB to linear (0.0 - 1.0)
                                        let linear = if level_db > -90.0 {
                                            10.0f64.powf(level_db / 20.0)
                                        } else {
                                            0.0
                                        };
                                        peak_levels.1 = linear;
                                    }
                                } else {
                                    // If mono, use the same value for both channels
                                    peak_levels.1 = peak_levels.0;
                                }
                                
                                // Store the peak levels
                                // In a real implementation, we would update the track's peak_levels field
                                // but since this is a callback, we would need to use Arc<Mutex<>> or similar
                                // to safely update the field from this thread
                                debug!("Track {} levels: L={:.2}, R={:.2}", track_id, peak_levels.0, peak_levels.1);
                            }
                        }
                    }
                }
            }
            None
        });
        
        self.level_watch_id = Some(level_watch_id);
        
        Ok(())
    }
    
    /// Play the audio track
    pub fn play(&mut self) -> Result<(), EditingError> {
        if self.pipeline.is_none() {
            self.initialize()?;
        }
        
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to playing state".to_string()))?;
            
            self.state = PlaybackState::Playing;
        }
        
        Ok(())
    }
    
    /// Pause the audio track
    pub fn pause(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to paused state".to_string()))?;
            
            self.state = PlaybackState::Paused;
        }
        
        Ok(())
    }
    
    /// Stop the audio track
    pub fn stop(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Ready)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to ready state".to_string()))?;
            
            self.state = PlaybackState::Stopped;
        }
        
        Ok(())
    }
    
    /// Set the volume level (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f64) -> Result<(), EditingError> {
        let volume = volume.max(0.0).min(1.0);
        self.volume_level = volume;
        
        if let Some(volume_element) = &self.volume {
            volume_element.set_property("volume", volume);
        }
        
        Ok(())
    }
    
    /// Set the pan position (-1.0 left to 1.0 right)
    pub fn set_pan(&mut self, pan: f64) -> Result<(), EditingError> {
        let pan = pan.max(-1.0).min(1.0);
        self.pan_position = pan;
        
        if let Some(pan_element) = &self.pan {
            pan_element.set_property("panorama", pan);
        }
        
        Ok(())
    }
    
    /// Set the mute state
    pub fn set_mute(&mut self, mute: bool) -> Result<(), EditingError> {
        self.muted = mute;
        
        if let Some(volume_element) = &self.volume {
            volume_element.set_property("mute", mute);
        }
        
        Ok(())
    }
    
    /// Set the solo state
    pub fn set_solo(&mut self, solo: bool) -> Result<(), EditingError> {
        self.solo = solo;
        
        Ok(())
    }
    
    /// Get the current playback position in seconds
    pub fn position(&self) -> Result<f64, EditingError> {
        if let Some(pipeline) = &self.pipeline {
            let position = pipeline.query_position::<gst::ClockTime>()
                .map(|pos| pos.seconds() as f64 / 1_000_000_000.0)
                .unwrap_or(0.0);
            
            Ok(position)
        } else {
            Ok(0.0)
        }
    }
    
    /// Get the duration in seconds
    pub fn duration(&self) -> Result<f64, EditingError> {
        if let Some(pipeline) = &self.pipeline {
            let duration = pipeline.query_duration::<gst::ClockTime>()
                .map(|dur| dur.seconds() as f64 / 1_000_000_000.0)
                .unwrap_or(0.0);
            
            Ok(duration)
        } else {
            Ok(0.0)
        }
    }
    
    /// Seek to the specified position in seconds
    pub fn seek(&self, position: f64) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            let position_ns = (position * 1_000_000_000.0) as u64;
            
            pipeline.seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                position_ns.nseconds(),
            ).map_err(|_| EditingError::AudioError("Failed to seek".to_string()))?;
            
            Ok(())
        } else {
            Err(EditingError::AudioError("Pipeline not initialized".to_string()))
        }
    }
    
    /// Add an audio effect to the track
    pub fn add_effect(&mut self, effect_type: AudioEffectType) -> Result<(), EditingError> {
        if self.audio_bin.is_none() {
            self.initialize()?;
        }
        
        let audio_bin = self.audio_bin.as_ref().unwrap();
        
        // Create the effect element based on the effect type
        let effect_element = match &effect_type {
            AudioEffectType::Equalizer { bands, gains } => {
                if bands.len() != gains.len() {
                    return Err(EditingError::AudioError("Number of bands must match number of gains".to_string()));
                }
                
                // Create equalizer element
                let equalizer = gst::ElementFactory::make("equalizer-nbands")
                    .name(&format!("eq-{}-{}", self.id, self.effects.len()))
                    .property("num-bands", bands.len() as i32)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create equalizer element".to_string()))?;
                
                // Set band frequencies and gains
                for (i, (freq, gain)) in bands.iter().zip(gains.iter()).enumerate() {
                    equalizer.set_property(&format!("band{}-freq", i), freq);
                    equalizer.set_property(&format!("band{}-gain", i), gain);
                }
                
                equalizer
            },
            AudioEffectType::Reverb { room_size, damping, wet_level, dry_level } => {
                // Create freeverb element
                let reverb = gst::ElementFactory::make("freeverb")
                    .name(&format!("reverb-{}-{}", self.id, self.effects.len()))
                    .property("room-size", room_size)
                    .property("damping", damping)
                    .property("level", wet_level)
                    .property("dry", dry_level)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create reverb element".to_string()))?;
                
                reverb
            },
            AudioEffectType::Delay { time_ms, feedback, mix } => {
                // Create delay element
                let delay = gst::ElementFactory::make("ladspa-delay")
                    .name(&format!("delay-{}-{}", self.id, self.effects.len()))
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create delay element".to_string()))?;
                
                // Set delay properties
                let delay_seconds = *time_ms as f64 / 1000.0;
                delay.set_property("delay-time", delay_seconds);
                delay.set_property("feedback", feedback);
                delay.set_property("dry-wet", mix);
                
                delay
            },
            AudioEffectType::Compressor { threshold, ratio, attack, release, makeup } => {
                // Create compressor element
                let compressor = gst::ElementFactory::make("audiodynamic")
                    .name(&format!("comp-{}-{}", self.id, self.effects.len()))
                    .property("mode", 1) // Compressor mode
                    .property("threshold", threshold)
                    .property("ratio", ratio)
                    .property("attack", attack / 1000.0) // Convert ms to seconds
                    .property("release", release / 1000.0) // Convert ms to seconds
                    .property("makeup", *makeup > 0.0) // Enable makeup gain
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create compressor element".to_string()))?;
                
                compressor
            },
        };
        
        // Find the last element in the chain before the resample element
        let last_effect = if !self.effects.is_empty() {
            &self.effects[self.effects.len() - 1]
        } else {
            self.pan.as_ref().unwrap()
        };
        
        // Find the resample element
        let resample = audio_bin.by_name(&format!("resample-{}", self.id)).unwrap();
        
        // Unlink the last effect from the resample element
        last_effect.unlink(&resample);
        
        // Add the new effect to the bin
        audio_bin.add(&effect_element)
            .map_err(|_| EditingError::AudioError("Failed to add effect to bin".to_string()))?;
        
        // Link the last effect to the new effect
        last_effect.link(&effect_element)
            .map_err(|_| EditingError::AudioError("Failed to link last effect to new effect".to_string()))?;
        
        // Link the new effect to the resample element
        effect_element.link(&resample)
            .map_err(|_| EditingError::AudioError("Failed to link new effect to resample".to_string()))?;
        
        // Sync state with parent
        effect_element.sync_state_with_parent()
            .map_err(|_| EditingError::AudioError("Failed to sync effect state with parent".to_string()))?;
        
        // Store the effect
        self.effects.push(effect_element);
        
        Ok(())
    }
    
    /// Remove an audio effect from the track
    pub fn remove_effect(&mut self, index: usize) -> Result<(), EditingError> {
        if index >= self.effects.len() {
            return Err(EditingError::AudioError(format!("Effect index {} out of bounds", index)));
        }
        
        if self.audio_bin.is_none() {
            return Err(EditingError::AudioError("Track not initialized".to_string()));
        }
        
        let audio_bin = self.audio_bin.as_ref().unwrap();
        
        // Get the effect to remove
        let effect = &self.effects[index];
        
        // Find the element before the effect
        let prev_element = if index > 0 {
            &self.effects[index - 1]
        } else {
            self.pan.as_ref().unwrap()
        };
        
        // Find the element after the effect
        let next_element = if index < self.effects.len() - 1 {
            &self.effects[index + 1]
        } else {
            audio_bin.by_name(&format!("resample-{}", self.id)).unwrap()
        };
        
        // Unlink the effect
        prev_element.unlink(effect);
        effect.unlink(next_element);
        
        // Link the previous element to the next element
        prev_element.link(next_element)
            .map_err(|_| EditingError::AudioError("Failed to link elements after removing effect".to_string()))?;
        
        // Remove the effect from the bin
        audio_bin.remove(effect)
            .map_err(|_| EditingError::AudioError("Failed to remove effect from bin".to_string()))?;
        
        // Remove the effect from the list
        self.effects.remove(index);
        
        Ok(())
    }
    
    /// Clear all audio effects from the track
    pub fn clear_effects(&mut self) -> Result<(), EditingError> {
        if self.audio_bin.is_none() {
            return Ok(());
        }
        
        // Remove effects in reverse order
        while !self.effects.is_empty() {
            self.remove_effect(self.effects.len() - 1)?;
        }
        
        Ok(())
    }
    
    /// Get the list of effects
    pub fn get_effects(&self) -> &[gst::Element] {
        &self.effects
    }
    
    /// Get the current peak levels (RMS) for left and right channels
    pub fn get_peak_levels(&self) -> (f64, f64) {
        self.peak_levels
    }
    
    /// Update the peak levels from the level meter element
    pub fn update_peak_levels(&mut self) -> Result<(f64, f64), EditingError> {
        if let Some(level) = &self.level {
            // In a real implementation, we would query the level element for the current peak values
            // For now, we'll just return the stored values
            Ok(self.peak_levels)
        } else {
            Err(EditingError::AudioError("Level meter not initialized".to_string()))
        }
    }
}

/// Helper function to handle pad-added signals
fn handle_pad_added(bin: &gst::Bin, src_pad: &gst::Pad) {
    // Check if the pad is an audio pad
    let caps = src_pad.current_caps().unwrap();
    let structure = caps.structure(0).unwrap();
    
    if structure.name().starts_with("audio/") {
        // Find the first sink pad of the volume element
        if let Some(volume) = bin.by_name(&format!("volume-{}", bin.name().unwrap())) {
            let sink_pad = volume.static_pad("sink").unwrap();
            
            // Link the pads
            src_pad.link(&sink_pad).unwrap();
        }
    }
}

/// Audio device information
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device name
    pub name: String,
    /// Device description
    pub description: String,
    /// Device ID
    pub id: String,
    /// Whether this is an input device
    pub is_input: bool,
    /// Whether this is the default device
    pub is_default: bool,
    /// Number of channels
    pub channels: u32,
    /// Sample rate
    pub sample_rate: u32,
}

/// Audio engine configuration
#[derive(Debug, Clone)]
pub struct AudioEngineConfig {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Buffer size in frames
    pub buffer_size: u32,
    /// Number of channels (1 for mono, 2 for stereo)
    pub channels: u32,
    /// Output device ID
    pub output_device: Option<String>,
    /// Input device ID
    pub input_device: Option<String>,
}

impl Default for AudioEngineConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            buffer_size: 1024,
            channels: 2,
            output_device: None,
            input_device: None,
        }
    }
}

/// Main audio engine
pub struct AudioEngine {
    /// Audio engine configuration
    config: AudioEngineConfig,
    /// Map of audio tracks by ID
    tracks: HashMap<String, Arc<Mutex<AudioTrack>>>,
    /// Master volume level (0.0 - 1.0)
    master_volume: f64,
    /// Whether the engine is initialized
    initialized: bool,
    /// Main GStreamer pipeline
    pipeline: Option<gst::Pipeline>,
    /// Audio mixer element
    mixer: Option<gst::Element>,
    /// Master volume element
    master_volume_element: Option<gst::Element>,
    /// Available audio devices
    devices: Vec<AudioDevice>,
    /// Bus watch ID for cleanup
    bus_watch_id: Option<glib::SourceId>,
}

impl AudioEngine {
    /// Create a new audio engine with default configuration
    pub fn new() -> Result<Self, EditingError> {
        Self::with_config(AudioEngineConfig::default())
    }
    
    /// Create a new audio engine with the given configuration
    pub fn with_config(config: AudioEngineConfig) -> Result<Self, EditingError> {
        // Initialize GStreamer if not already initialized
        if !gst::is_initialized() {
            gst::init().map_err(|e| EditingError::AudioError(format!("Failed to initialize GStreamer: {}", e)))?;
        }
        
        Ok(Self {
            config,
            tracks: HashMap::new(),
            master_volume: 1.0,
            initialized: false,
            pipeline: None,
            mixer: None,
            master_volume_element: None,
            devices: Vec::new(),
            bus_watch_id: None,
        })
    }
    
    /// Initialize the audio engine
    pub fn initialize(&mut self) -> Result<(), EditingError> {
        if self.initialized {
            return Ok(());
        }
        
        // Create the main pipeline
        let pipeline = gst::Pipeline::new(Some("audio-engine"));
        
        // Create the audio mixer
        let mixer = gst::ElementFactory::make("audiomixer")
            .name("audio-mixer")
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create audio mixer".to_string()))?;
        
        // Create the master volume element
        let volume = gst::ElementFactory::make("volume")
            .name("master-volume")
            .build()
            .map_err(|_| EditingError::AudioError("Failed to create master volume element".to_string()))?;
        
        // Create the audio sink
        let sink = if let Some(device_id) = &self.config.output_device {
            // Use the specified output device
            gst::ElementFactory::make("autoaudiosink")
                .name("audio-sink")
                .property("device", device_id)
                .build()
                .map_err(|_| EditingError::AudioError("Failed to create audio sink".to_string()))?
        } else {
            // Use the default output device
            gst::ElementFactory::make("autoaudiosink")
                .name("audio-sink")
                .build()
                .map_err(|_| EditingError::AudioError("Failed to create audio sink".to_string()))?
        };
        
        // Add elements to the pipeline
        pipeline.add_many(&[&mixer, &volume, &sink])
            .map_err(|_| EditingError::AudioError("Failed to add elements to pipeline".to_string()))?;
        
        // Link elements
        mixer.link(&volume)
            .map_err(|_| EditingError::AudioError("Failed to link mixer to volume".to_string()))?;
        
        volume.link(&sink)
            .map_err(|_| EditingError::AudioError("Failed to link volume to sink".to_string()))?;
        
        // Set up bus watch
        let bus = pipeline.bus().expect("Pipeline has no bus");
        let bus_watch_id = bus.add_watch(move |_, msg| {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    error!("Audio engine error: {} ({})", err.error(), err.debug().unwrap_or_default());
                },
                gst::MessageView::Eos(_) => {
                    debug!("End of stream reached");
                },
                _ => (),
            }
            
            glib::Continue(true)
        }).map_err(|_| EditingError::AudioError("Failed to add bus watch".to_string()))?;
        
        // Set the pipeline to ready state
        pipeline.set_state(gst::State::Ready)
            .map_err(|_| EditingError::AudioError("Failed to set pipeline to ready state".to_string()))?;
        
        // Store the elements
        self.pipeline = Some(pipeline);
        self.mixer = Some(mixer);
        self.master_volume_element = Some(volume);
        self.bus_watch_id = Some(bus_watch_id);
        
        // Refresh the device list
        self.refresh_devices()?;
        
        self.initialized = true;
        
        Ok(())
    }
    
    /// Add a new audio track to the engine
    pub fn add_track(&mut self, id: &str, source: AudioSourceType) -> Result<(), EditingError> {
        if !self.initialized {
            self.initialize()?;
        }
        
        // Check if a track with this ID already exists
        if self.tracks.contains_key(id) {
            return Err(EditingError::AudioError(format!("Track with ID '{}' already exists", id)));
        }
        
        // Create a new track
        let mut track = AudioTrack::new(id, source);
        
        // Initialize the track
        track.initialize()?;
        
        // Get the track's audio bin
        let audio_bin = track.audio_bin.as_ref().unwrap();
        
        // Get the mixer element
        let mixer = self.mixer.as_ref().unwrap();
        
        // Get the pipeline
        let pipeline = self.pipeline.as_ref().unwrap();
        
        // Add the track's bin to the main pipeline
        pipeline.add(audio_bin)
            .map_err(|_| EditingError::AudioError("Failed to add track bin to pipeline".to_string()))?;
        
        // Get the src pad from the track's bin
        let src_pad = audio_bin.static_pad("src").unwrap();
        
        // Get a request pad from the mixer
        let mixer_pad = mixer.request_pad_simple("sink_%u").unwrap();
        
        // Link the track's bin to the mixer
        src_pad.link(&mixer_pad)
            .map_err(|_| EditingError::AudioError("Failed to link track to mixer".to_string()))?;
        
        // Store the track
        self.tracks.insert(id.to_string(), Arc::new(Mutex::new(track)));
        
        Ok(())
    }
    
    /// Remove an audio track from the engine
    pub fn remove_track(&mut self, id: &str) -> Result<(), EditingError> {
        if let Some(track) = self.tracks.remove(id) {
            let mut track = track.lock().unwrap();
            
            // Stop the track
            track.stop()?;
            
            // Get the track's audio bin
            if let Some(audio_bin) = &track.audio_bin {
                // Get the pipeline
                if let Some(pipeline) = &self.pipeline {
                    // Remove the bin from the pipeline
                    pipeline.remove(audio_bin)
                        .map_err(|_| EditingError::AudioError("Failed to remove track bin from pipeline".to_string()))?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Get a track by ID
    pub fn get_track(&self, id: &str) -> Option<Arc<Mutex<AudioTrack>>> {
        self.tracks.get(id).cloned()
    }
    
    /// Get all tracks
    pub fn get_tracks(&self) -> Vec<Arc<Mutex<AudioTrack>>> {
        self.tracks.values().cloned().collect()
    }
    
    /// Play all tracks
    pub fn play(&mut self) -> Result<(), EditingError> {
        if !self.initialized {
            self.initialize()?;
        }
        
        // Start the pipeline
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to playing state".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Pause all tracks
    pub fn pause(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to paused state".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Stop all tracks
    pub fn stop(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Ready)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to ready state".to_string()))?;
        }
        
        Ok(())
    }
    
    /// Refresh the list of available audio devices
    pub fn refresh_devices(&mut self) -> Result<(), EditingError> {
        // Create a device monitor
        let monitor = gst::DeviceMonitor::new();
        
        // Add filters for audio devices
        monitor.add_filter(Some("Audio/Source"), None);
        monitor.add_filter(Some("Audio/Sink"), None);
        
        // Start the monitor
        if !monitor.start() {
            return Err(EditingError::AudioError("Failed to start device monitor".to_string()));
        }
        
        // Get the devices
        let devices = monitor.devices();
        
        // Stop the monitor
        monitor.stop();
        
        // Clear the current device list
        self.devices.clear();
        
        // Process the devices
        for device in devices {
            let props = device.properties().unwrap();
            
            // Get device information
            let name = props.get::<String>("device.description")
                .unwrap_or_else(|_| device.display_name().to_string());
            
            let device_class = props.get::<String>("device.class")
                .unwrap_or_default();
            
            let is_input = device_class.contains("source");
            let is_default = props.get::<bool>("device.is_default")
                .unwrap_or(false);
            
            // Get device ID
            let id = props.get::<String>("device.path")
                .or_else(|_| props.get::<String>("device.id"))
                .unwrap_or_else(|_| format!("device-{}", self.devices.len()));
            
            // Get device capabilities
            let caps = device.caps().unwrap();
            let mut channels = 2;
            let mut sample_rate = 48000;
            
            // Try to get channel and sample rate information from caps
            for i in 0..caps.size() {
                let structure = caps.structure(i).unwrap();
                
                if structure.name().starts_with("audio/") {
                    // Get channels
                    if let Ok(ch) = structure.get::<i32>("channels") {
                        channels = ch as u32;
                    }
                    
                    // Get sample rate
                    if let Ok(rate) = structure.get::<i32>("rate") {
                        sample_rate = rate as u32;
                    }
                    
                    break;
                }
            }
            
            // Create the device
            let audio_device = AudioDevice {
                name,
                description: device_class,
                id,
                is_input,
                is_default,
                channels,
                sample_rate,
            };
            
            // Add the device to the list
            self.devices.push(audio_device);
        }
        
        // If no devices were found, add default devices
        if self.devices.is_empty() {
            self.devices = vec![
                AudioDevice {
                    name: "Default Output".to_string(),
                    description: "System default output device".to_string(),
                    id: "default".to_string(),
                    is_input: false,
                    is_default: true,
                    channels: 2,
                    sample_rate: 48000,
                },
                AudioDevice {
                    name: "Default Input".to_string(),
                    description: "System default input device".to_string(),
                    id: "default-input".to_string(),
                    is_input: true,
                    is_default: true,
                    channels: 2,
                    sample_rate: 48000,
                },
            ];
        }
        
        Ok(())
    }
    
    /// Get a list of available audio devices
    pub fn get_devices(&self) -> &[AudioDevice] {
        &self.devices
    }
    
    /// Set the master volume level (0.0 - 1.0)
    pub fn set_master_volume(&mut self, volume: f64) -> Result<(), EditingError> {
        let volume = volume.max(0.0).min(1.0);
        self.master_volume = volume;
        
        if let Some(volume_element) = &self.master_volume_element {
            volume_element.set_property("volume", volume);
        }
        
        Ok(())
    }
    
    /// Get the master volume level
    pub fn master_volume(&self) -> f64 {
        self.master_volume
    }
    
    /// Shutdown the audio engine
    pub fn shutdown(&mut self) -> Result<(), EditingError> {
        if !self.initialized {
            return Ok(());
        }
        
        // Stop all tracks
        for (_, track) in &self.tracks {
            let mut track = track.lock().unwrap();
            
            // Remove level watch
            if let Some(watch_id) = track.level_watch_id.take() {
                watch_id.remove();
            }
            
            // Stop the pipeline
            if let Some(pipeline) = &track.pipeline {
                let _ = pipeline.set_state(gst::State::Null);
            }
        }
        
        // Stop the main pipeline
        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
        
        // Remove the bus watch
        if let Some(watch_id) = self.bus_watch_id.take() {
            watch_id.remove();
        }
        
        self.initialized = false;
        
        Ok(())
    }
    
    /// Set the output device
    pub fn set_output_device(&mut self, device_id: &str) -> Result<(), EditingError> {
        // Update the configuration
        self.config.output_device = Some(device_id.to_string());
        
        // If the engine is already initialized, we need to update the sink
        if self.initialized {
            if let Some(pipeline) = &self.pipeline {
                // Get the current sink
                let old_sink = pipeline.by_name("audio-sink").unwrap();
                
                // Create a new sink with the specified device
                let new_sink = gst::ElementFactory::make("autoaudiosink")
                    .name("audio-sink")
                    .property("device", device_id)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create audio sink".to_string()))?;
                
                // Get the volume element
                let volume = self.master_volume_element.as_ref().unwrap();
                
                // Unlink the volume from the old sink
                volume.unlink(&old_sink);
                
                // Add the new sink to the pipeline
                pipeline.add(&new_sink)
                    .map_err(|_| EditingError::AudioError("Failed to add new sink to pipeline".to_string()))?;
                
                // Link the volume to the new sink
                volume.link(&new_sink)
                    .map_err(|_| EditingError::AudioError("Failed to link volume to new sink".to_string()))?;
                
                // Sync the new sink's state with the pipeline
                new_sink.sync_state_with_parent()
                    .map_err(|_| EditingError::AudioError("Failed to sync new sink state with parent".to_string()))?;
                
                // Remove the old sink from the pipeline
                pipeline.remove(&old_sink)
                    .map_err(|_| EditingError::AudioError("Failed to remove old sink from pipeline".to_string()))?;
            }
        }
        
        Ok(())
    }
    
    /// Get the current output device ID
    pub fn get_output_device(&self) -> Option<&str> {
        self.config.output_device.as_deref()
    }
    
    /// Get a device by ID
    pub fn get_device_by_id(&self, id: &str) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.id == id)
    }
    
    /// Get the default output device
    pub fn get_default_output_device(&self) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| !d.is_input && d.is_default)
    }
    
    /// Get the default input device
    pub fn get_default_input_device(&self) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.is_input && d.is_default)
    }
    
    /// Get all output devices
    pub fn get_output_devices(&self) -> Vec<&AudioDevice> {
        self.devices.iter().filter(|d| !d.is_input).collect()
    }
    
    /// Get all input devices
    pub fn get_input_devices(&self) -> Vec<&AudioDevice> {
        self.devices.iter().filter(|d| d.is_input).collect()
    }
}
