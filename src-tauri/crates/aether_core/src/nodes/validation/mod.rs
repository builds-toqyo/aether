use crate::nodes::{NodeError, NodeResult};
use aether_types::{Graph, Node, Connection, PinDataType, ParameterValue};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

mod connection_validator;
mod node_validator;
mod graph_validator;
mod type_checker;

pub use connection_validator::ConnectionValidator;
pub use node_validator::NodeValidator;
pub use graph_validator::GraphValidator;
pub use type_checker::TypeChecker;
