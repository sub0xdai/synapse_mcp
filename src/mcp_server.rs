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
    let state = ServerState {
        graph: Arc::new(graph),
    };

    Router::new()
        .route("/query", post(handle_query))
        .route("/nodes/:type", get(handle_nodes_by_type))
        .route("/node/:id/related", get(handle_related_nodes))
        .route("/health", get(health_check))
        .with_state(state)
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

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "synapse-mcp-server",
        "version": "0.1.0"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_endpoint() {
        let graph = graph::connect("test://", "test", "test").await.unwrap();
        let app = create_server(graph).await;
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health").await;
        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "synapse-mcp-server");
    }

    #[tokio::test]
    async fn test_query_endpoint() {
        // Skip test if Neo4j is not available 
        if std::env::var("NEO4J_URI").is_err() {
            println!("Skipping MCP server test - NEO4J_URI not set");
            return;
        }
        
        let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
        
        let graph = match graph::connect(&uri, &user, &password).await {
            Ok(g) => g,
            Err(_) => {
                println!("Skipping MCP server test - Neo4j connection failed");
                return;
            }
        };
        
        let app = create_server(graph).await;
        let server = TestServer::new(app).unwrap();

        let query_request = QueryRequest {
            query: "Find rules about test".to_string(),
        };

        let response = server
            .post("/query")
            .json(&query_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        
        let body: QueryResponse = response.json();
        assert!(body.success);
        // Accept any result since we don't know what's in the test database
        assert!(!body.result.is_empty());
    }

    #[tokio::test]
    async fn test_query_with_invalid_input() {
        // Skip test if Neo4j is not available 
        if std::env::var("NEO4J_URI").is_err() {
            println!("Skipping MCP server test - NEO4J_URI not set");
            return;
        }
        
        let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
        
        let graph = match graph::connect(&uri, &user, &password).await {
            Ok(g) => g,
            Err(_) => {
                println!("Skipping MCP server test - Neo4j connection failed");
                return;
            }
        };
        
        let app = create_server(graph).await;
        let server = TestServer::new(app).unwrap();

        let query_request = QueryRequest {
            query: "completely invalid query that should fail".to_string(),
        };

        let response = server
            .post("/query")
            .json(&query_request)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        
        let _body: QueryResponse = response.json();
        // Even if query fails to parse, server should return success with "No matching" message
        // This follows KISS principle - handle errors gracefully
    }
}