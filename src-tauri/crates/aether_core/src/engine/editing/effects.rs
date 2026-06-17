use std::collections::HashMap;
use anyhow::Result;
use gstreamer as gst;
use gst::prelude::*;
use gstreamer_editing_services as ges;
use crate::engine::editing::types::EditingError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectType {
    ColorCorrection,
    ColorGrading,
    Blur,
    Sharpen,
    Crop,
    Scale,
    Rotate,
    Flip,
    Text,
    Overlay,

    Volume,
    Fade,
    Equalizer,
    Reverb,
    Delay,

    Custom(String),
}

impl EffectType {
    pub fn to_gst_name(&self) -> &str {
        match self {
            EffectType::ColorCorrection => "videobalance",
            EffectType::ColorGrading => "videoconvert ! glcolorbalance",
            EffectType::Blur => "gaussianblur",
            EffectType::Sharpen => "unsharp",
            EffectType::Crop => "videocrop",
            EffectType::Scale => "videoscale",
            EffectType::Rotate => "videoflip",
            EffectType::Flip => "videoflip",
            EffectType::Text => "textoverlay",
            EffectType::Overlay => "compositor",
            EffectType::Volume => "volume",
            EffectType::Fade => "volume",
            EffectType::Equalizer => "equalizer-10bands",
            EffectType::Reverb => "audioecho",
            EffectType::Delay => "audiodelay",
            EffectType::Custom(name) => name,
        }
    }


    pub fn default_parameters(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        match self {
            EffectType::ColorCorrection => {
                params.insert("brightness".to_string(), "0.0".to_string());
                params.insert("contrast".to_string(), "1.0".to_string());
                params.insert("saturation".to_string(), "1.0".to_string());
                params.insert("hue".to_string(), "0.0".to_string());
            },
            EffectType::Blur => {
                params.insert("sigma".to_string(), "1.0".to_string());
            },
            EffectType::Volume => {
                params.insert("volume".to_string(), "1.0".to_string());
            },
            EffectType::Text => {
                params.insert("text".to_string(), "Text".to_string());
                params.insert("font-desc".to_string(), "Sans 24".to_string());
                params.insert("valignment".to_string(), "center".to_string());
                params.insert("halignment".to_string(), "center".to_string());
            },
            _ => {}
        }

        params
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionType {
    Crossfade,
    Wipe,
    Slide,
    Fade,

    AudioCrossfade,

    Custom(String),
}

impl TransitionType {

    pub fn to_gst_name(&self) -> &str {
        match self {
            TransitionType::Crossfade => "crossfade",
            TransitionType::Wipe => "wipe",
            TransitionType::Slide => "slide",
            TransitionType::Fade => "fade",
            TransitionType::AudioCrossfade => "audiomixer",
            TransitionType::Custom(name) => name,
        }
    }
}

pub struct Effect {
    pub effect_type: EffectType,

    pub parameters: HashMap<String, String>,

    ges_effect: Option<ges::Effect>,
}

impl Effect {
    pub fn new(effect_type: EffectType) -> Self {
        let parameters = effect_type.default_parameters();

        Self {
            effect_type,
            parameters,
            ges_effect: None,
        }
    }

    pub fn set_parameter(&mut self, name: &str, value: &str) -> Result<(), EditingError> {
        self.parameters.insert(name.to_string(), value.to_string());

        if let Some(effect) = &self.ges_effect {
            effect.set_property_from_str(name, value);
        }

        Ok(())
    }

    pub fn create_ges_effect(&mut self) -> Result<ges::Effect, EditingError> {
        let effect_name = self.effect_type.to_gst_name();
        let effect = ges::Effect::new(effect_name)?;

        for (name, value) in &self.parameters {
            effect.set_property_from_str(name, value);
        }

        self.ges_effect = Some(effect.clone());

        Ok(effect)
    }
}

pub struct Transition {
    pub transition_type: TransitionType,

    pub parameters: HashMap<String, String>,

    ges_transition: Option<ges::TransitionClip>,
}

impl Transition {
    pub fn new(transition_type: TransitionType) -> Self {
        Self {
            transition_type,
            parameters: HashMap::new(),
            ges_transition: None,
        }
    }

    pub fn set_parameter(&mut self, name: &str, value: &str) -> Result<(), EditingError> {
        self.parameters.insert(name.to_string(), value.to_string());

        if let Some(transition) = &self.ges_transition {
            transition.set_property_from_str(name, value);
        }

        Ok(())
    }

    fn transition_type_to_enum(&self) -> ges::VideoStandardTransitionType {
        match self.transition_type {
            TransitionType::Crossfade => ges::VideoStandardTransitionType::Crossfade,
            TransitionType::Wipe => ges::VideoStandardTransitionType::BarWipeLr,
            TransitionType::Slide => ges::VideoStandardTransitionType::BoxWipeTl,
            TransitionType::Fade => ges::VideoStandardTransitionType::Crossfade,
            TransitionType::AudioCrossfade => ges::VideoStandardTransitionType::Crossfade,
            TransitionType::Custom(ref name) => match name.as_str() {
                "crossfade" => ges::VideoStandardTransitionType::Crossfade,
                "fade" => ges::VideoStandardTransitionType::Crossfade,
                "wipe" => ges::VideoStandardTransitionType::BarWipeLr,
                "slide" => ges::VideoStandardTransitionType::BoxWipeTl,
                _ => ges::VideoStandardTransitionType::Crossfade,
            },
        }
    }

    pub fn create_ges_transition(&mut self, _track_type: ges::TrackType) -> Result<ges::TransitionClip, EditingError> {
        let vtype = self.transition_type_to_enum();
        let transition = ges::TransitionClip::new(vtype)
            .ok_or_else(|| EditingError::EffectError(format!("Failed to create {:?} transition clip", vtype)))?;

        for (name, value) in &self.parameters {
            transition.set_property_from_str(name, value);
        }

        self.ges_transition = Some(transition.clone());
        Ok(transition)
    }
}
