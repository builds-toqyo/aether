use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin, BlendMode};
use uuid::Uuid;

mod blend_ops;
mod node;

pub use blend_ops::BlendOperations;
pub use node::MergeNode;
