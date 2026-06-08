use crate::types::{ColorSpace};

#[derive(Debug, Clone)]
pub struct AcesConfig {
    pub use_opencolorio: bool,
    pub ocio_config_path: String,
    pub working_color_space: ColorSpace,
    pub default_input_transform: super::InputTransform,
    pub default_output_transform: super::OutputTransform,
}

impl Default for AcesConfig {
    fn default() -> Self {
        Self {
            use_opencolorio: true,
            ocio_config_path: "aces_1.3/config.ocio".to_string(),
            working_color_space: ColorSpace::Rec709,
            default_input_transform: super::InputTransform::Rec709ToAces,
            default_output_transform: super::OutputTransform::AcesToRec709,
        }
    }
}

impl AcesConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_opencolorio(mut self, use_opencolorio: bool) -> Self {
        self.use_opencolorio = use_opencolorio;
        self
    }

    pub fn with_config_path(mut self, path: String) -> Self {
        self.ocio_config_path = path;
        self
    }

    pub fn with_working_color_space(mut self, color_space: ColorSpace) -> Self {
        self.working_color_space = color_space;
        self
    }

    pub fn with_default_input_transform(mut self, transform: super::InputTransform) -> Self {
        self.default_input_transform = transform;
        self
    }

    pub fn with_default_output_transform(mut self, transform: super::OutputTransform) -> Self {
        self.default_output_transform = transform;
        self
    }
}
