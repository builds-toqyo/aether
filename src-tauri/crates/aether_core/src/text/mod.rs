//! Text animation system
//! 
//! This module provides comprehensive text animation capabilities including
//! typography controls, text-on-path support, and character-by-character animation.

pub mod types;
pub mod typography;
pub mod animation;
pub mod path_text;
pub mod renderer;

// Re-export main text types
pub use types::{TextLayer, TextContent, TextStyle, TextAlignment, TextDirection};
pub use typography::{TypographyControls, FontMetrics, TextLayout};
pub use animation::{TextAnimator, CharacterAnimation, AnimationType, TextKeyframe};
pub use path_text::{TextOnPath, PathTextRenderer};
pub use renderer::{TextRenderer, GlyphRenderer};
