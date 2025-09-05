// Re-export all functionality from the new module structure
pub mod pattern_enforcer;

pub use pattern_enforcer::{
    PatternEnforcer,
    EnforceCheckRequest,
    EnforceCheckResponse, 
    EnforceContextRequest,
    EnforceContextResponse,
    RulesForPathRequest,
    RulesForPathResponse,
    RuleViolation,
    RuleContextInfo,
};

use crate::{graph, Result, SynapseError, NodeType};
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

pub async fn start_server(graph: graph::Graph, port: u16) -> Result<()> {
    let app = create_server(graph).await;
    let addr = format!("0.0.0.0:{}", port);
    
    println!("Starting Synapse MCP server on {}", addr);
    
    let listener = TcpListener::bind(&addr).await
        .map_err(|e| SynapseError::Io(e))?;
        
    axum::serve(listener, app).await
        .map_err(|e| SynapseError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(())
}

pub async fn start_server_with_enforcer(
    graph: graph::Graph, 
    enforcer: Option<PatternEnforcer>,
    port: u16
) -> Result<()> {
    let has_enforcer = enforcer.is_some();
    let app = create_server_with_enforcer(graph, enforcer).await;
    let addr = format!("0.0.0.0:{}", port);
    
    println!("Starting Synapse MCP server with PatternEnforcer on {}", addr);
    if has_enforcer {
        println!("✅ Rule enforcement endpoints enabled");
    }
    
    let listener = TcpListener::bind(&addr).await
        .map_err(|e| SynapseError::Io(e))?;
        
    axum::serve(listener, app).await
        .map_err(|e| SynapseError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
    
    Ok(())
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
    Json(request): Json<EnforceCheckRequest>,
) -> Json<EnforceCheckResponse> {
    match &state.enforcer {
        Some(enforcer) => {
            match enforcer.check_files(request) {
                Ok(response) => Json(response),
                Err(e) => Json(EnforceCheckResponse {
                    success: false,
                    violations: Vec::new(),
                    files_checked: 0,
                    rules_applied: 0,
                    error: Some(e.to_string()),
                }),
            }
        }
        None => Json(EnforceCheckResponse {
            success: false,
            violations: Vec::new(),
            files_checked: 0,
            rules_applied: 0,
            error: Some("PatternEnforcer not available".to_string()),
        }),
    }
}

async fn handle_enforce_context(
    State(state): State<ServerState>,
    Json(request): Json<EnforceContextRequest>,
) -> Json<EnforceContextResponse> {
    match &state.enforcer {
        Some(enforcer) => {
            match enforcer.generate_context(request) {
                Ok(response) => Json(response),
                Err(e) => Json(EnforceContextResponse {
                    success: false,
                    context: None,
                    applicable_rules: Vec::new(),
                    inheritance_chain: Vec::new(),
                    overridden_rules: Vec::new(),
                    error: Some(e.to_string()),
                }),
            }
        }
        None => Json(EnforceContextResponse {
            success: false,
            context: None,
            applicable_rules: Vec::new(),
            inheritance_chain: Vec::new(),
            overridden_rules: Vec::new(),
            error: Some("PatternEnforcer not available".to_string()),
        }),
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
                Err(e) => Json(RulesForPathResponse {
                    success: false,
                    path: std::path::PathBuf::new(),
                    rules: Vec::new(),
                    inheritance_chain: Vec::new(),
                    overridden_rules: Vec::new(),
                    error: Some(e.to_string()),
                }),
            }
        }
        None => Json(RulesForPathResponse {
            success: false,
            path: std::path::PathBuf::new(),
            rules: Vec::new(),
            inheritance_chain: Vec::new(),
            overridden_rules: Vec::new(),
            error: Some("PatternEnforcer not available".to_string()),
        }),
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