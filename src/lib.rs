pub mod models;
pub mod indexer;
pub mod mcp_server;
pub mod graph;
pub mod error;
pub mod rules;
pub mod rule_graph;
pub mod enforcement;
pub mod api_models;
pub mod formatting;
pub mod config;


pub use models::{Node, Edge, NodeType, EdgeType, Rule, RuleSet, RuleNode, CompositeRules, RuleType, CompiledRule, PatternMatcher, Violation};
pub use error::{SynapseError, Result};
pub use rule_graph::{RuleGraph, RuleGraphStats};
pub use indexer::parse_markdown_file;
pub use rules::{RuleSystem};
pub use mcp_server::{PatternEnforcer};
pub use enforcement::check_rules;
pub use api_models::{
    ApiRequest, ApiResponse, CheckRequest, CheckResponse, ContextRequest, ContextResponse,
    RulesForPathRequest, RulesForPathResponse, RuleViolationDto, RuleContextInfo,
    CheckData, CheckResultData, ContextData, ContextResultData, RulesForPathData, RulesForPathResultData
};
pub use formatting::{
    OutputFormatter, Formattable, MarkdownFormatter, JsonFormatter, PlainFormatter,
    get_formatter, FormattableContext
};
pub use config::{Config, Neo4jConfig, ServerConfig, RuntimeConfig};

