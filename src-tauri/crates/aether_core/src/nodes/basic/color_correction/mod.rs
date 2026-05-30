use aether_types::{Node, NodeType, ParameterValue, PinDataType, InputPin, OutputPin};
use uuid::Uuid;
use log::debug;

pub mod types;
mod processor;
mod gpu_ops;
mod node;

pub use types::*;
pub use processor::ColorProcessor;
pub use gpu_ops::GpuOperations;
pub use node::ColorCorrectionNode;
