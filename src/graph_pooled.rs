//! Refactored Neo4j graph operations with connection pooling
//! 
//! This module provides the same interface as graph.rs but uses
//! connection pooling for better performance and resource management.

use crate::{
    Node, Edge, NodeType, EdgeType, Result, SynapseError,
    ConnectionPool, PoolError, Neo4jConfig
};
use std::env;
use tracing::{debug, info, warn, error, instrument};

/// Graph database operations with connection pooling
/// 
/// This struct maintains backward compatibility with the original Graph
/// but uses connection pooling for better resource management.
pub struct PooledGraph {
    pool: ConnectionPool,
    verbose: bool,
}

impl std::fmt::Debug for PooledGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PooledGraph")
            .field("pool", &"ConnectionPool<..>")
            .field("verbose", &self.verbose)
            .finish()
    }
}

impl PooledGraph {
    /// Create a new pooled graph from configuration
    #[instrument(skip(neo4j_config))]
    pub async fn new(neo4j_config: Neo4jConfig) -> Result<Self> {
        info!("Creating pooled graph with connection pooling");
        
        let connection_config = neo4j_config.to_connection_config();
        let pool_config = neo4j_config.pool.clone();
        
        let pool = ConnectionPool::new(connection_config, pool_config)
            .await
            .map_err(|e| match e {
                PoolError::PoolCreation(bb8_err) => {
                    error!("Failed to create connection pool: {}", bb8_err);
                    SynapseError::Database(format!("Connection pool creation failed: {}", bb8_err))
                }
                PoolError::Configuration(msg) => {
                    error!("Pool configuration error: {}", msg);
                    SynapseError::Configuration(msg)
                }
                _ => {
                    error!("Unexpected pool error: {}", e);
                    SynapseError::Database(format!("Pool error: {}", e))
                }
            })?;
        
        let verbose = env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true";
        
        info!("Successfully created pooled graph");
        Ok(Self { pool, verbose })
    }
    
    /// Create a pooled graph with custom pool
    pub fn with_pool(pool: ConnectionPool) -> Self {
        let verbose = env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true";
        Self { pool, verbose }
    }
    
    /// Get pool statistics
    pub async fn pool_stats(&self) -> crate::PoolStats {
        self.pool.stats().await
    }
    
    /// Check if the pool is healthy
    pub async fn health_check(&self) -> Result<bool> {
        match self.pool.health_check().await {
            Ok(healthy) => Ok(healthy),
            Err(e) => {
                warn!("Pool health check failed: {}", e);
                Ok(false)
            }
        }
    }
    
    /// Get a connection from the pool for direct operations
    pub async fn get_connection(&self) -> Result<bb8::PooledConnection<'_, crate::Neo4jConnectionManager>> {
        self.pool.get_connection().await.map_err(|e| {
            error!("Failed to get connection from pool: {}", e);
            match e {
                PoolError::Timeout => SynapseError::Database("Connection pool timeout".to_string()),
                PoolError::GetConnection(msg) => SynapseError::Database(msg),
                _ => SynapseError::Database(format!("Pool error: {}", e)),
            }
        })
    }
}

// Maintain backward compatibility - create functions that work with PooledGraph

/// Create a node using connection pool
#[instrument(skip(graph, node), fields(node_id = %node.id, node_label = %node.label))]
pub async fn create_node_pooled(graph: &PooledGraph, node: &Node) -> Result<()> {
    node.validate()?;
    
    let query = "
        MERGE (n { id: $id })
        ON CREATE SET n.created_at = timestamp()
        SET n.label = $label,
            n.content = $content,
            n.node_type = $node_type,
            n.tags = $tags,
            n.updated_at = timestamp()
        RETURN n
    ";
    
    let tags_json = serde_json::to_string(&node.tags).unwrap_or_else(|_| "[]".to_string());
    
    let conn = graph.get_connection().await?;
    let mut result = conn.execute(
        neo4rs::query(query)
            .param("id", node.id.clone())
            .param("label", node.label.clone())
            .param("content", node.content.clone())
            .param("node_type", format!("{:?}", node.node_type))
            .param("tags", tags_json)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if graph.verbose {
        debug!("Created/updated node: {} ({})", node.label, node.id);
    }
    
    result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
    Ok(())
}

/// Create an edge using connection pool
#[instrument(skip(graph, edge), fields(source_id = %edge.source_id, target_id = %edge.target_id))]
pub async fn create_edge_pooled(graph: &PooledGraph, edge: &Edge) -> Result<()> {
    edge.validate()?;
    
    let relationship_type = edge_type_to_relationship(&edge.edge_type);
    
    let query = format!("
        MATCH (source {{ id: $source_id }}), (target {{ id: $target_id }})
        MERGE (source)-[r:{} {{}}]->(target)
        ON CREATE SET r.created_at = timestamp()
        SET r.label = $label,
            r.edge_type = $edge_type,
            r.updated_at = timestamp()
        RETURN r
    ", relationship_type);
    
    let conn = graph.get_connection().await?;
    let mut result = conn.execute(
        neo4rs::query(&query)
            .param("source_id", edge.source_id.clone())
            .param("target_id", edge.target_id.clone())
            .param("label", edge.label.clone())
            .param("edge_type", format!("{:?}", edge.edge_type))
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if graph.verbose {
        debug!("Created/updated edge: {} -> {} ({})", edge.source_id, edge.target_id, edge.label);
    }
    
    result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
    Ok(())
}

/// Query nodes by type using connection pool
#[instrument(skip(graph), fields(node_type = ?node_type))]
pub async fn query_nodes_by_type_pooled(graph: &PooledGraph, node_type: &NodeType) -> Result<Vec<Node>> {
    let query = "
        MATCH (n { node_type: $node_type })
        RETURN n.id as id, n.label as label, n.content as content, 
               n.node_type as node_type, n.tags as tags
        ORDER BY n.label
    ";
    
    let conn = graph.get_connection().await?;
    let mut result = conn.execute(
        neo4rs::query(query)
            .param("node_type", format!("{:?}", node_type))
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    let mut nodes = Vec::new();
    
    while let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
        let id: String = row.get("id").unwrap_or_default();
        let label: String = row.get("label").unwrap_or_default();
        let content: String = row.get("content").unwrap_or_default();
        let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
        
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        
        let mut node = Node::new(node_type.clone(), label, content);
        node.id = id;
        node.tags = tags;
        
        nodes.push(node);
    }
    
    debug!("Found {} nodes of type {:?}", nodes.len(), node_type);
    Ok(nodes)
}

/// Find related nodes using connection pool
#[instrument(skip(graph), fields(node_id = %node_id))]
pub async fn find_related_nodes_pooled(graph: &PooledGraph, node_id: &str) -> Result<Vec<(Node, Edge)>> {
    let query = "
        MATCH (n { id: $node_id })-[r]->(related)
        RETURN related.id as id, related.label as label, related.content as content,
               related.node_type as node_type, related.tags as tags,
               r.label as edge_label, r.edge_type as edge_type
        UNION
        MATCH (n { id: $node_id })<-[r]-(related)
        RETURN related.id as id, related.label as label, related.content as content,
               related.node_type as node_type, related.tags as tags,
               r.label as edge_label, r.edge_type as edge_type
    ";
    
    let conn = graph.get_connection().await?;
    let mut result = conn.execute(
        neo4rs::query(query)
            .param("node_id", node_id)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    let mut relationships = Vec::new();
    
    while let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
        let id: String = row.get("id").unwrap_or_default();
        let label: String = row.get("label").unwrap_or_default();
        let content: String = row.get("content").unwrap_or_default();
        let node_type_str: String = row.get("node_type").unwrap_or_default();
        let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
        let edge_label: String = row.get("edge_label").unwrap_or_default();
        let edge_type_str: String = row.get("edge_type").unwrap_or_default();
        
        // Parse node type
        let node_type = match node_type_str.as_str() {
            "Rule" => NodeType::Rule,
            "Decision" => NodeType::Decision,
            "Architecture" => NodeType::Architecture,
            "Component" => NodeType::Component,
            "Function" => NodeType::Function,
            _ => NodeType::File,
        };
        
        // Parse edge type
        let edge_type = match edge_type_str.as_str() {
            "RelatesTo" => EdgeType::RelatesTo,
            "ImplementsRule" => EdgeType::ImplementsRule,
            "DefinedIn" => EdgeType::DefinedIn,
            "Inherits" => EdgeType::Inherits,
            "Overrides" => EdgeType::Overrides,
            "DependsOn" => EdgeType::DependsOn,
            "Contains" => EdgeType::Contains,
            "References" => EdgeType::References,
            _ => EdgeType::RelatesTo,
        };
        
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        
        let mut node = Node::new(node_type, label, content);
        node.id = id.clone();
        node.tags = tags;
        
        let edge = Edge::new(
            node_id.to_string(), 
            id,
            edge_type,
            edge_label,
        );
        
        relationships.push((node, edge));
    }
    
    debug!("Found {} related nodes for node_id {}", relationships.len(), node_id);
    Ok(relationships)
}

/// Delete a node using connection pool
#[instrument(skip(graph), fields(node_id = %node_id))]
pub async fn delete_node_pooled(graph: &PooledGraph, node_id: &str) -> Result<()> {
    let query = "
        MATCH (n { id: $node_id })
        DETACH DELETE n
    ";
    
    let conn = graph.get_connection().await?;
    let mut result = conn.execute(
        neo4rs::query(query)
            .param("node_id", node_id)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if graph.verbose {
        debug!("Deleted node: {}", node_id);
    }
    
    result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
    Ok(())
}

/// Execute a custom Cypher query using connection pool  
#[instrument(skip(graph, query), fields(query_preview = %format!("{}...", &query[..query.len().min(50)])))]
pub async fn execute_query_pooled(graph: &PooledGraph, query: &str) -> Result<String> {
    let conn = graph.get_connection().await?;
    let mut result = conn.execute(neo4rs::query(query)).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    let mut results = Vec::new();
    let mut row_count = 0;
    
    while let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
        let mut record_parts = Vec::new();
        
        // Extract values as strings for simplicity
        for key in &["id", "label", "content", "node_type", "count", "name"] {
            if let Ok(value) = row.get::<String>(key) {
                record_parts.push(format!("{}: {}", key, value));
            } else if let Ok(value) = row.get::<i64>(key) {
                record_parts.push(format!("{}: {}", key, value));
            }
        }
        
        if !record_parts.is_empty() {
            results.push(format!("{{ {} }}", record_parts.join(", ")));
        }
        row_count += 1;
    }
    
    debug!("Query returned {} results", row_count);
    Ok(if results.is_empty() {
        format!("Query executed successfully, {} rows returned", row_count)
    } else {
        results.join("\n")
    })
}

// Helper function (copied from original)
fn edge_type_to_relationship(edge_type: &EdgeType) -> &'static str {
    match edge_type {
        EdgeType::RelatesTo => "RELATES_TO",
        EdgeType::ImplementsRule => "IMPLEMENTS_RULE", 
        EdgeType::DefinedIn => "DEFINED_IN",
        EdgeType::Inherits => "INHERITS",
        EdgeType::Overrides => "OVERRIDES",
        EdgeType::DependsOn => "DEPENDS_ON",
        EdgeType::Contains => "CONTAINS",
        EdgeType::References => "REFERENCES",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Config;
    
    #[tokio::test]
    #[ignore] // Run only with --ignored when Neo4j is available
    async fn test_pooled_graph_operations() {
        // Skip test if Neo4j is not available
        if std::env::var("NEO4J_URI").is_err() {
            println!("Skipping pooled graph test - NEO4J_URI not set");
            return;
        }
        
        let config = Config::for_testing();
        
        let graph = match PooledGraph::new(config.neo4j).await {
            Ok(g) => g,
            Err(e) => {
                println!("Skipping pooled graph test - Neo4j connection failed: {}", e);
                return;
            }
        };
        
        // Test health check
        let health = graph.health_check().await.unwrap();
        assert!(health, "Graph should be healthy");
        
        // Test basic operations
        let node = Node::new(
            NodeType::Rule,
            "Test Pooled Rule".to_string(),
            "Test pooled content".to_string(),
        );
        
        // Test creation
        assert!(create_node_pooled(&graph, &node).await.is_ok());
        
        // Test querying
        let nodes = query_nodes_by_type_pooled(&graph, &NodeType::Rule).await.unwrap();
        assert!(!nodes.is_empty());
        
        // Test pool stats
        let stats = graph.pool_stats().await;
        assert!(stats.size > 0);
        assert!(stats.max_size > 0);
        
        // Clean up test node
        let _ = delete_node_pooled(&graph, &node.id).await;
    }
    
    #[tokio::test]
    async fn test_pool_configuration() {
        let config = Config::for_testing();
        
        // This test focuses on configuration, not actual connection
        assert_eq!(config.neo4j.pool.min_idle, 1);
        assert_eq!(config.neo4j.pool.max_size, 5);
        assert_eq!(config.neo4j.pool.connection_timeout_secs, 10);
        
        let connection_config = config.neo4j.to_connection_config();
        assert_eq!(connection_config.uri, "bolt://localhost:7687");
        assert_eq!(connection_config.user, "test");
    }
}