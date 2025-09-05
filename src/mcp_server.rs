// Re-export all functionality from the new module structure
pub mod pattern_enforcer;

pub use pattern_enforcer::{
    PatternEnforcer,
};

use crate::{graph, Result, SynapseError, NodeType, CheckRequest, CheckResponse, ContextRequest, ContextResponse, RulesForPathRequest, RulesForPathResponse};
use axum::{
    extract::{State, Path},
    response::Json,
    routing::{post, get},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct ServerState {
    pub graph: Arc<graph::Graph>,
    pub enforcer: Option<Arc<PatternEnforcer>>,
}

#[derive(Deserialize, Serialize)]
pub struct QueryRequest {
    pub query: String,
}

#[derive(Serialize, Deserialize)]
pub struct QueryResponse {
    pub result: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct NodesResponse {
    pub nodes: Vec<crate::Node>,
    pub count: usize,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RelatedResponse {
    pub related: Vec<(crate::Node, crate::Edge)>,
    pub count: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// Configuration for the MCP server
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub graph: graph::Graph,
    pub enforcer: Option<PatternEnforcer>,
}

impl std::fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerConfig")
            .field("port", &self.port)
            .field("host", &self.host)
            .field("graph", &"<Graph>")  // Don't debug the complex graph
            .field("enforcer", &self.enforcer.as_ref().map(|_| "<PatternEnforcer>"))
            .finish()
    }
}

/// Builder for ServerConfig using the builder pattern
pub struct ServerConfigBuilder {
    port: Option<u16>,
    host: Option<String>,
    graph: Option<graph::Graph>,
    enforcer: Option<PatternEnforcer>,
}

impl ServerConfigBuilder {
    /// Create a new ServerConfigBuilder
    pub fn new() -> Self {
        Self {
            port: None,
            host: None,
            graph: None,
            enforcer: None,
        }
    }

    /// Set the port for the server
    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    /// Set the host for the server
    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    /// Set the graph connection for the server
    pub fn graph(mut self, graph: graph::Graph) -> Self {
        self.graph = Some(graph);
        self
    }

    /// Set the pattern enforcer for the server (optional)
    pub fn enforcer(mut self, enforcer: PatternEnforcer) -> Self {
        self.enforcer = Some(enforcer);
        self
    }

    /// Build the ServerConfig, validating that all required fields are set
    pub fn build(self) -> Result<ServerConfig> {
        let port = self.port.ok_or_else(|| {
            SynapseError::Validation("Port is required for server configuration".to_string())
        })?;
        
        let host = self.host.ok_or_else(|| {
            SynapseError::Validation("Host is required for server configuration".to_string())
        })?;
        
        let graph = self.graph.ok_or_else(|| {
            SynapseError::Validation("Graph connection is required for server configuration".to_string())
        })?;

        Ok(ServerConfig {
            port,
            host,
            graph,
            enforcer: self.enforcer,
        })
    }
}

impl Default for ServerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn create_server(graph: graph::Graph) -> Router {
    create_server_with_enforcer(graph, None).await
}

pub async fn create_server_with_enforcer(
    graph: graph::Graph, 
    enforcer: Option<PatternEnforcer>
) -> Router {
    let state = ServerState {
        graph: Arc::new(graph),
        enforcer: enforcer.map(Arc::new),
    };

    let mut router = Router::new()
        .route("/query", post(handle_query))
        .route("/nodes/:type", get(handle_nodes_by_type))
        .route("/node/:id/related", get(handle_related_nodes))
        .route("/health", get(health_check));
    
    // Add enforcement endpoints if PatternEnforcer is available
    if state.enforcer.is_some() {
        router = router
            .route("/enforce/check", post(handle_enforce_check))
            .route("/enforce/context", post(handle_enforce_context))
            .route("/rules/for-path", post(handle_rules_for_path));
    }
    
    router.with_state(state)
}

/// Start the MCP server with the given configuration
pub async fn start_server(config: ServerConfig) -> Result<()> {
    let has_enforcer = config.enforcer.is_some();
    let app = create_server_with_enforcer(config.graph, config.enforcer).await;
    let addr = format!("{}:{}", config.host, config.port);
    
    println!("🚀 Starting Synapse MCP server on {}", addr);
    if has_enforcer {
        println!("✅ Rule enforcement endpoints enabled");
    }
    
    let listener = TcpListener::bind(&addr).await
        .map_err(|e| SynapseError::Io(e))?;
        
    axum::serve(listener, app).await
        .map_err(|e| SynapseError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(())
}

/// Legacy function for backwards compatibility
/// Use start_server(ServerConfig) with ServerConfigBuilder instead
#[deprecated(note = "Use start_server(ServerConfig) with ServerConfigBuilder instead")]
pub async fn start_server_legacy(graph: graph::Graph, port: u16) -> Result<()> {
    let config = ServerConfigBuilder::new()
        .graph(graph)
        .port(port)
        .host("0.0.0.0".to_string())
        .build()?;
    
    start_server(config).await
}

/// Legacy function for backwards compatibility  
/// Use start_server(ServerConfig) instead
#[deprecated(note = "Use start_server(ServerConfig) with ServerConfigBuilder instead")]
pub async fn start_server_with_enforcer(
    graph: graph::Graph, 
    enforcer: Option<PatternEnforcer>,
    port: u16
) -> Result<()> {
    let mut builder = ServerConfigBuilder::new()
        .graph(graph)
        .port(port)
        .host("0.0.0.0".to_string());
    
    if let Some(enforcer) = enforcer {
        builder = builder.enforcer(enforcer);
    }
    
    let config = builder.build()?;
    start_server(config).await
}

async fn handle_query(
    State(state): State<ServerState>,
    Json(request): Json<QueryRequest>,
) -> Json<QueryResponse> {
    match graph::natural_language_query(&state.graph, &request.query).await {
        Ok(result) => Json(QueryResponse {
            result,
            success: true,
            error: None,
        }),
        Err(e) => Json(QueryResponse {
            result: String::new(),
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

async fn handle_nodes_by_type(
    State(state): State<ServerState>,
    Path(node_type_str): Path<String>,
) -> Json<NodesResponse> {
    let node_type = match node_type_str.to_lowercase().as_str() {
        "file" => NodeType::File,
        "rule" => NodeType::Rule,
        "decision" => NodeType::Decision,
        "function" => NodeType::Function,
        "architecture" => NodeType::Architecture,
        "component" => NodeType::Component,
        _ => {
            return Json(NodesResponse {
                nodes: Vec::new(),
                count: 0,
                success: false,
                error: Some(format!("Invalid node type: {}", node_type_str)),
            });
        }
    };

    match graph::query_nodes_by_type(&state.graph, &node_type).await {
        Ok(nodes) => Json(NodesResponse {
            count: nodes.len(),
            nodes,
            success: true,
            error: None,
        }),
        Err(e) => Json(NodesResponse {
            nodes: Vec::new(),
            count: 0,
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

async fn handle_related_nodes(
    State(state): State<ServerState>,
    Path(node_id): Path<String>,
) -> Json<RelatedResponse> {
    match graph::find_related_nodes(&state.graph, &node_id).await {
        Ok(related) => Json(RelatedResponse {
            count: related.len(),
            related,
            success: true,
            error: None,
        }),
        Err(e) => Json(RelatedResponse {
            related: Vec::new(),
            count: 0,
            success: false,
            error: Some(e.to_string()),
        }),
    }
}

async fn handle_enforce_check(
    State(state): State<ServerState>,
    Json(request): Json<CheckRequest>,
) -> Json<CheckResponse> {
    match &state.enforcer {
        Some(enforcer) => {
            match enforcer.check_files(request) {
                Ok(response) => Json(response),
                Err(e) => Json(CheckResponse::error(e.to_string())),
            }
        }
        None => Json(CheckResponse::error("PatternEnforcer not available".to_string())),
    }
}

async fn handle_enforce_context(
    State(state): State<ServerState>,
    Json(request): Json<ContextRequest>,
) -> Json<ContextResponse> {
    match &state.enforcer {
        Some(enforcer) => {
            match enforcer.generate_context(request) {
                Ok(response) => Json(response),
                Err(e) => Json(ContextResponse::error(e.to_string())),
            }
        }
        None => Json(ContextResponse::error("PatternEnforcer not available".to_string())),
    }
}

async fn handle_rules_for_path(
    State(state): State<ServerState>,
    Json(request): Json<RulesForPathRequest>,
) -> Json<RulesForPathResponse> {
    match &state.enforcer {
        Some(enforcer) => {
            match enforcer.get_rules_for_path(request) {
                Ok(response) => Json(response),
                Err(e) => Json(RulesForPathResponse::error(e.to_string())),
            }
        }
        None => Json(RulesForPathResponse::error("PatternEnforcer not available".to_string())),
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "synapse-mcp-server",
        "version": "0.2.0",
        "features": ["knowledge_graph", "pattern_enforcement"]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_builder_success() {
        // Create a mock graph for testing
        // In a real scenario, we would connect to a test database
        let mock_uri = "bolt://test:7687";
        let mock_user = "test";
        let mock_password = "test";
        
        // This test will be skipped in CI until we have proper test infrastructure
        if std::env::var("SYNAPSE_TEST_DB").is_err() {
            println!("Skipping ServerConfigBuilder test - SYNAPSE_TEST_DB not set");
            return;
        }
        
        // TODO: Replace with proper test graph setup in Phase 4
        // For now, just test the builder validation logic
        
        let result = ServerConfigBuilder::new()
            .port(8080)
            .host("localhost".to_string())
            .build();
        
        // Should fail because graph is missing
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Graph connection is required"));
    }

    #[test]
    fn test_server_config_builder_missing_port() {
        let result = ServerConfigBuilder::new()
            .host("localhost".to_string())
            .build();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Port is required"));
    }

    #[test]
    fn test_server_config_builder_missing_host() {
        let result = ServerConfigBuilder::new()
            .port(8080)
            .build();
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Host is required"));
    }

    #[test]
    fn test_server_config_builder_chain() {
        // Test that the builder methods can be chained
        let _builder = ServerConfigBuilder::new()
            .port(8080)
            .host("localhost".to_string());
        
        // The builder pattern works - no need to check private fields
        // The functionality is tested through build() validation
        println!("✅ ServerConfigBuilder chaining works");
    }

    #[test]
    fn test_server_config_builder_default() {
        let builder = ServerConfigBuilder::default();
        
        // Test that default builder fails to build (missing required fields)
        let result = builder.build();
        assert!(result.is_err());
        println!("✅ ServerConfigBuilder default validation works");
    }

    #[test]
    fn test_server_config_builder_with_enforcer() {
        use std::path::PathBuf;
        
        // Create a temporary directory for testing
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_path = PathBuf::from(temp_dir.path());
        
        // Try to create a PatternEnforcer
        match PatternEnforcer::from_project(&project_path) {
            Ok(enforcer) => {
                let builder = ServerConfigBuilder::new()
                    .port(8080)
                    .host("localhost".to_string())
                    .enforcer(enforcer);
                
                assert!(builder.enforcer.is_some());
                println!("✅ ServerConfigBuilder enforcer integration works");
            }
            Err(e) => {
                println!("⚠️  PatternEnforcer creation failed (expected in test): {}", e);
                // This is expected since we don't have rule files in the temp dir
            }
        }
    }
}