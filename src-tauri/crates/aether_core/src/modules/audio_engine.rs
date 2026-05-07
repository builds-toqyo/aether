use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use log::{debug, info, warn, error};
use gst::prelude::*;
use glib;

use crate::engine::editing::types::EditingError;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {

    Stopped,

    Playing,

    Paused,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioSourceType {

    File(PathBuf),

    Uri(String),

    Raw(Vec<u8>, String),
}


#[derive(Debug, Clone)]
pub enum AudioEffectType {

    Equalizer {

        bands: Vec<f64>,

        gains: Vec<f64>,
    },

    Reverb {

        room_size: f64,

        damping: f64,

        wet_level: f64,

        dry_level: f64,
    },

    Delay {

        time_ms: u64,

        feedback: f64,

        mix: f64,
    },

    Compressor {

        threshold: f64,

        ratio: f64,

        attack: f64,

        release: f64,

        makeup: f64,
    },
}


pub struct AudioTrack {

    id: String,

    source: AudioSourceType,

    pipeline: Option<gst::Pipeline>,

    audio_bin: Option<gst::Bin>,

    volume: Option<gst::Element>,

    pan: Option<gst::Element>,

    level: Option<gst::Element>,

    volume_level: f64,

    pan_position: f64,

    muted: bool,

    soloed: bool,

    playback_state: PlaybackState,

    effects: Vec<gst::Element>,

    peak_levels: (f64, f64),

    level_watch_id: Option<glib::SourceId>,
}

impl AudioTrack {

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


    pub fn pause(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to paused state".to_string()))?;

            self.state = PlaybackState::Paused;
        }

        Ok(())
    }


    pub fn stop(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Ready)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to ready state".to_string()))?;

            self.state = PlaybackState::Stopped;
        }

        Ok(())
    }


    pub fn set_volume(&mut self, volume: f64) -> Result<(), EditingError> {
        let volume = volume.max(0.0).min(1.0);
        self.volume_level = volume;

        if let Some(volume_element) = &self.volume {
            volume_element.set_property("volume", volume);
        }

        Ok(())
    }


    pub fn set_pan(&mut self, pan: f64) -> Result<(), EditingError> {
        let pan = pan.max(-1.0).min(1.0);
        self.pan_position = pan;

        if let Some(pan_element) = &self.pan {
            pan_element.set_property("panorama", pan);
        }

        Ok(())
    }


    pub fn set_mute(&mut self, mute: bool) -> Result<(), EditingError> {
        self.muted = mute;

        if let Some(volume_element) = &self.volume {
            volume_element.set_property("mute", mute);
        }

        Ok(())
    }


    pub fn set_solo(&mut self, solo: bool) -> Result<(), EditingError> {
        self.solo = solo;

        Ok(())
    }


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


    pub fn add_effect(&mut self, effect_type: AudioEffectType) -> Result<(), EditingError> {
        if self.audio_bin.is_none() {
            self.initialize()?;
        }

        let audio_bin = self.audio_bin.as_ref().unwrap();


        let effect_element = match &effect_type {
            AudioEffectType::Equalizer { bands, gains } => {
                if bands.len() != gains.len() {
                    return Err(EditingError::AudioError("Number of bands must match number of gains".to_string()));
                }


                let equalizer = gst::ElementFactory::make("equalizer-nbands")
                    .name(&format!("eq-{}-{}", self.id, self.effects.len()))
                    .property("num-bands", bands.len() as i32)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create equalizer element".to_string()))?;


                for (i, (freq, gain)) in bands.iter().zip(gains.iter()).enumerate() {
                    equalizer.set_property(&format!("band{}-freq", i), freq);
                    equalizer.set_property(&format!("band{}-gain", i), gain);
                }

                equalizer
            },
            AudioEffectType::Reverb { room_size, damping, wet_level, dry_level } => {

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

                let delay = gst::ElementFactory::make("ladspa-delay")
                    .name(&format!("delay-{}-{}", self.id, self.effects.len()))
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create delay element".to_string()))?;


                let delay_seconds = *time_ms as f64 / 1000.0;
                delay.set_property("delay-time", delay_seconds);
                delay.set_property("feedback", feedback);
                delay.set_property("dry-wet", mix);

                delay
            },
            AudioEffectType::Compressor { threshold, ratio, attack, release, makeup } => {

                let compressor = gst::ElementFactory::make("audiodynamic")
                    .name(&format!("comp-{}-{}", self.id, self.effects.len()))
                    .property("mode", 1)
                    .property("threshold", threshold)
                    .property("ratio", ratio)
                    .property("attack", attack / 1000.0)
                    .property("release", release / 1000.0)
                    .property("makeup", *makeup > 0.0)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create compressor element".to_string()))?;

                compressor
            },
        };


        let last_effect = if !self.effects.is_empty() {
            &self.effects[self.effects.len() - 1]
        } else {
            self.pan.as_ref().unwrap()
        };


        let resample = audio_bin.by_name(&format!("resample-{}", self.id)).unwrap();


        last_effect.unlink(&resample);


        audio_bin.add(&effect_element)
            .map_err(|_| EditingError::AudioError("Failed to add effect to bin".to_string()))?;


        last_effect.link(&effect_element)
            .map_err(|_| EditingError::AudioError("Failed to link last effect to new effect".to_string()))?;


        effect_element.link(&resample)
            .map_err(|_| EditingError::AudioError("Failed to link new effect to resample".to_string()))?;


        effect_element.sync_state_with_parent()
            .map_err(|_| EditingError::AudioError("Failed to sync effect state with parent".to_string()))?;


        self.effects.push(effect_element);

        Ok(())
    }


    pub fn remove_effect(&mut self, index: usize) -> Result<(), EditingError> {
        if index >= self.effects.len() {
            return Err(EditingError::AudioError(format!("Effect index {} out of bounds", index)));
        }

        if self.audio_bin.is_none() {
            return Err(EditingError::AudioError("Track not initialized".to_string()));
        }

        let audio_bin = self.audio_bin.as_ref().unwrap();


        let effect = &self.effects[index];


        let prev_element = if index > 0 {
            &self.effects[index - 1]
        } else {
            self.pan.as_ref().unwrap()
        };


        let next_element = if index < self.effects.len() - 1 {
            &self.effects[index + 1]
        } else {
            audio_bin.by_name(&format!("resample-{}", self.id)).unwrap()
        };


        prev_element.unlink(effect);
        effect.unlink(next_element);


        prev_element.link(next_element)
            .map_err(|_| EditingError::AudioError("Failed to link elements after removing effect".to_string()))?;


        audio_bin.remove(effect)
            .map_err(|_| EditingError::AudioError("Failed to remove effect from bin".to_string()))?;


        self.effects.remove(index);

        Ok(())
    }


    pub fn clear_effects(&mut self) -> Result<(), EditingError> {
        if self.audio_bin.is_none() {
            return Ok(());
        }


        while !self.effects.is_empty() {
            self.remove_effect(self.effects.len() - 1)?;
        }

        Ok(())
    }


    pub fn get_effects(&self) -> &[gst::Element] {
        &self.effects
    }


    pub fn get_peak_levels(&self) -> (f64, f64) {
        self.peak_levels
    }


    pub fn update_peak_levels(&mut self) -> Result<(f64, f64), EditingError> {
        if let Some(level) = &self.level {


        let audio_bin = track.audio_bin.as_ref().unwrap();


        let mixer = self.mixer.as_ref().unwrap();


        let pipeline = self.pipeline.as_ref().unwrap();


        let src_pad = audio_bin.static_pad("src").unwrap();


        let mixer_pad = mixer.request_pad_simple("sink_%u").unwrap();


            if let Some(audio_bin) = &track.audio_bin {

                if let Some(pipeline) = &self.pipeline {

                    pipeline.remove(audio_bin)
                        .map_err(|_| EditingError::AudioError("Failed to remove track bin from pipeline".to_string()))?;
                }
            }
        }

        Ok(())
    }


    pub fn get_track(&self, id: &str) -> Option<Arc<Mutex<AudioTrack>>> {
        self.tracks.get(id).cloned()
    }


    pub fn get_tracks(&self) -> Vec<Arc<Mutex<AudioTrack>>> {
        self.tracks.values().cloned().collect()
    }


    pub fn play(&mut self) -> Result<(), EditingError> {
        if !self.initialized {
            self.initialize()?;
        }


        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Playing)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to playing state".to_string()))?;
        }

        Ok(())
    }


    pub fn pause(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Paused)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to paused state".to_string()))?;
        }

        Ok(())
    }


    pub fn stop(&mut self) -> Result<(), EditingError> {
        if let Some(pipeline) = &self.pipeline {
            pipeline.set_state(gst::State::Ready)
                .map_err(|_| EditingError::AudioError("Failed to set pipeline to ready state".to_string()))?;
        }

        Ok(())
    }


    pub fn refresh_devices(&mut self) -> Result<(), EditingError> {

        let monitor = gst::DeviceMonitor::new();


        monitor.add_filter(Some("Audio/Source"), None);
        monitor.add_filter(Some("Audio/Sink"), None);


        if !monitor.start() {
            return Err(EditingError::AudioError("Failed to start device monitor".to_string()));
        }


        let devices = monitor.devices();


        monitor.stop();


        self.devices.clear();


        for device in devices {
            let props = device.properties().unwrap();


            let name = props.get::<String>("device.description")
                .unwrap_or_else(|_| device.display_name().to_string());

            let device_class = props.get::<String>("device.class")
                .unwrap_or_default();

            let is_input = device_class.contains("source");
            let is_default = props.get::<bool>("device.is_default")
                .unwrap_or(false);


            let id = props.get::<String>("device.path")
                .or_else(|_| props.get::<String>("device.id"))
                .unwrap_or_else(|_| format!("device-{}", self.devices.len()));


            let caps = device.caps().unwrap();
            let mut channels = 2;
            let mut sample_rate = 48000;


            for i in 0..caps.size() {
                let structure = caps.structure(i).unwrap();

                if structure.name().starts_with("audio/") {

                    if let Ok(ch) = structure.get::<i32>("channels") {
                        channels = ch as u32;
                    }


                    if let Ok(rate) = structure.get::<i32>("rate") {
                        sample_rate = rate as u32;
                    }

                    break;
                }
            }


            let audio_device = AudioDevice {
                name,
                description: device_class,
                id,
                is_input,
                is_default,
                channels,
                sample_rate,
            };


            self.devices.push(audio_device);
        }


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


    pub fn get_devices(&self) -> &[AudioDevice] {
        &self.devices
    }


    pub fn set_master_volume(&mut self, volume: f64) -> Result<(), EditingError> {
        let volume = volume.max(0.0).min(1.0);
        self.master_volume = volume;

        if let Some(volume_element) = &self.master_volume_element {
            volume_element.set_property("volume", volume);
        }

        Ok(())
    }


    pub fn master_volume(&self) -> f64 {
        self.master_volume
    }


    pub fn shutdown(&mut self) -> Result<(), EditingError> {
        if !self.initialized {
            return Ok(());
        }


        for (_, track) in &self.tracks {
            let mut track = track.lock().unwrap();


            if let Some(watch_id) = track.level_watch_id.take() {
                watch_id.remove();
            }


            if let Some(pipeline) = &track.pipeline {
                let _ = pipeline.set_state(gst::State::Null);
            }
        }


        if let Some(pipeline) = &self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }


        if let Some(watch_id) = self.bus_watch_id.take() {
            watch_id.remove();
        }

        self.initialized = false;

        Ok(())
    }


    pub fn set_output_device(&mut self, device_id: &str) -> Result<(), EditingError> {

        self.config.output_device = Some(device_id.to_string());


        if self.initialized {
            if let Some(pipeline) = &self.pipeline {

                let old_sink = pipeline.by_name("audio-sink").unwrap();


                let new_sink = gst::ElementFactory::make("autoaudiosink")
                    .name("audio-sink")
                    .property("device", device_id)
                    .build()
                    .map_err(|_| EditingError::AudioError("Failed to create audio sink".to_string()))?;


                let volume = self.master_volume_element.as_ref().unwrap();


                volume.unlink(&old_sink);


                pipeline.add(&new_sink)
                    .map_err(|_| EditingError::AudioError("Failed to add new sink to pipeline".to_string()))?;


                volume.link(&new_sink)
                    .map_err(|_| EditingError::AudioError("Failed to link volume to new sink".to_string()))?;


                new_sink.sync_state_with_parent()
                    .map_err(|_| EditingError::AudioError("Failed to sync new sink state with parent".to_string()))?;


                pipeline.remove(&old_sink)
                    .map_err(|_| EditingError::AudioError("Failed to remove old sink from pipeline".to_string()))?;
            }
        }

        Ok(())
    }


    pub fn get_output_device(&self) -> Option<&str> {
        self.config.output_device.as_deref()
    }


    pub fn get_device_by_id(&self, id: &str) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.id == id)
    }


    pub fn get_default_output_device(&self) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| !d.is_input && d.is_default)
    }


    pub fn get_default_input_device(&self) -> Option<&AudioDevice> {
        self.devices.iter().find(|d| d.is_input && d.is_default)
    }


    pub fn get_output_devices(&self) -> Vec<&AudioDevice> {
        self.devices.iter().filter(|d| !d.is_input).collect()
    }


    pub fn get_input_devices(&self) -> Vec<&AudioDevice> {
        self.devices.iter().filter(|d| d.is_input).collect()
    }
}
