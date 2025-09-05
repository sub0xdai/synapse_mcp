pub mod models;
pub mod indexer;
pub mod mcp_server;
pub mod graph;
pub mod error;
pub mod rules;
pub mod rule_graph;


pub use models::{Node, Edge, NodeType, EdgeType, Rule, RuleSet, RuleNode, CompositeRules, RuleType};
pub use error::{SynapseError, Result};
pub use rule_graph::{RuleGraph, RuleGraphStats};
pub use rules::{RuleSystem};
pub use mcp_server::{PatternEnforcer};

