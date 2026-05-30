pub mod animator;
pub mod layer;
pub mod path;
pub mod types;
pub mod typography;

pub use animator::{TextAnimator, CharacterAnimation};
pub use types::{AnimationType, TextKeyframe};

pub type AnimationValue = aether_types::animation::AnimationValue;
