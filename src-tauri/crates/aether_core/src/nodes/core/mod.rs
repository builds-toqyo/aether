use crate::nodes::{NodeExecutor, ExecutionContext, NodeError, NodeResult};
use aether_types::{Node, NodeType, PinDataType, ParameterValue};
use std::collections::HashMap;
use uuid::Uuid;

pub use super::basic::{
    InputNode, OutputNode, MergeNode, TransformNode, ColorCorrectionNode, BlurNode,
};

mod factory;
mod input_node;
mod output_node;
mod transform_node;
mod merge_node;

pub use factory::CoreNodes;
pub use input_node::CoreInputNode;
pub use output_node::CoreOutputNode;
pub use transform_node::CoreTransformNode;
pub use merge_node::CoreMergeNode;
