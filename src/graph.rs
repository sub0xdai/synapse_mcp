// Neo4j graph database operations with optional connection pooling
use crate::{Node, Edge, NodeType, EdgeType, Result, SynapseError};
use neo4rs::{Graph as Neo4jGraph, ConfigBuilder};
use std::env;

// Re-export pooled graph functionality for advanced users
pub use crate::graph_pooled::{PooledGraph, create_node_pooled, create_edge_pooled, query_nodes_by_type_pooled, find_related_nodes_pooled, delete_node_pooled, execute_query_pooled};

/// Connection provider abstraction for internal use
enum ConnectionProvider {
    Direct(Neo4jGraph),
    Pooled(crate::graph_pooled::PooledGraph),
}

impl ConnectionProvider {
    /// Execute a query and collect all results immediately
    async fn execute_query_all<T, F>(&self, query: neo4rs::Query, mut mapper: F) -> Result<Vec<T>>
    where
        F: FnMut(&neo4rs::Row) -> Result<T>,
    {
        match self {
            ConnectionProvider::Direct(client) => {
                let mut result = client.execute(query).await.map_err(|e| SynapseError::Neo4j(e))?;
                let mut items = Vec::new();
                
                while let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
                    items.push(mapper(&row)?);
                }
                
                Ok(items)
            }
            ConnectionProvider::Pooled(pooled) => {
                let conn = pooled.get_connection().await.map_err(|e| {
                    SynapseError::Database(format!("Failed to get pooled connection: {}", e))
                })?;
                
                let mut result = conn.execute(query).await.map_err(|e| SynapseError::Neo4j(e))?;
                let mut items = Vec::new();
                
                while let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
                    items.push(mapper(&row)?);
                }
                
                Ok(items)
            }
        }
    }
    
    /// Execute a query expecting a single result (or None)
    async fn execute_query_single<T, F>(&self, query: neo4rs::Query, mapper: F) -> Result<Option<T>>
    where
        F: FnOnce(&neo4rs::Row) -> Result<T>,
    {
        match self {
            ConnectionProvider::Direct(client) => {
                let mut result = client.execute(query).await.map_err(|e| SynapseError::Neo4j(e))?;
                
                if let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
                    Ok(Some(mapper(&row)?))
                } else {
                    Ok(None)
                }
            }
            ConnectionProvider::Pooled(pooled) => {
                let conn = pooled.get_connection().await.map_err(|e| {
                    SynapseError::Database(format!("Failed to get pooled connection: {}", e))
                })?;
                
                let mut result = conn.execute(query).await.map_err(|e| SynapseError::Neo4j(e))?;
                
                if let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
                    Ok(Some(mapper(&row)?))
                } else {
                    Ok(None)
                }
            }
        }
    }
    
    /// Execute a query that doesn't return data (like CREATE, DELETE)
    async fn execute_query_void(&self, query: neo4rs::Query) -> Result<()> {
        match self {
            ConnectionProvider::Direct(client) => {
                let mut result = client.execute(query).await.map_err(|e| SynapseError::Neo4j(e))?;
                // Consume the result to ensure the query is executed
                let _ = result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
                Ok(())
            }
            ConnectionProvider::Pooled(pooled) => {
                let conn = pooled.get_connection().await.map_err(|e| {
                    SynapseError::Database(format!("Failed to get pooled connection: {}", e))
                })?;
                
                let mut result = conn.execute(query).await.map_err(|e| SynapseError::Neo4j(e))?;
                // Consume the result to ensure the query is executed
                let _ = result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
                Ok(())
            }
        }
    }
}

pub struct Graph {
    provider: ConnectionProvider,
}

impl std::fmt::Debug for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.provider {
            ConnectionProvider::Direct(_) => f.debug_struct("Graph")
                .field("mode", &"Direct")
                .finish(),
            ConnectionProvider::Pooled(_) => f.debug_struct("Graph")
                .field("mode", &"Pooled")
                .finish(),
        }
    }
}

impl Graph {
    /// Create a new pooled graph (recommended)
    pub async fn new_pooled(neo4j_config: crate::Neo4jConfig) -> Result<Self> {
        let pooled = crate::graph_pooled::PooledGraph::new(neo4j_config).await?;
        Ok(Graph { 
            provider: ConnectionProvider::Pooled(pooled) 
        })
    }
    
    /// Create a direct connection graph (legacy)
    pub async fn new_direct(uri: &str, user: &str, password: &str) -> Result<Self> {
        let client = connect_direct(uri, user, password).await?;
        Ok(Graph {
            provider: ConnectionProvider::Direct(client)
        })
    }
    
    /// Simple health check query to verify database connectivity
    pub async fn health_check(&self) -> Result<bool> {
        use tracing::debug;
        match &self.provider {
            ConnectionProvider::Direct(client) => {
                match client.execute(neo4rs::query("RETURN 1 as health")).await {
                    Ok(_) => Ok(true),
                    Err(e) => {
                        debug!("Direct graph health check failed: {}", e);
                        Ok(false)
                    }
                }
            }
            ConnectionProvider::Pooled(pooled) => {
                pooled.health_check().await
            }
        }
    }
    
    /// Get pool statistics (only available for pooled connections)
    pub async fn pool_stats(&self) -> Option<crate::PoolStats> {
        match &self.provider {
            ConnectionProvider::Pooled(pooled) => Some(pooled.pool_stats().await),
            ConnectionProvider::Direct(_) => None,
        }
    }
}

impl Graph {
    #[cfg(test)]
    pub fn new_mock() -> Result<Self> {
        // For testing, we create a mock graph that doesn't actually connect to Neo4j
        // This will be replaced with a proper test database setup in Phase 4
        use neo4rs::ConfigBuilder;
        
        // Create a placeholder config - in real tests this would be a test DB
        let _config = ConfigBuilder::default()
            .uri("bolt://localhost:7687")
            .user("test")
            .password("test")
            .db("test")
            .build()
            .map_err(|e| SynapseError::Neo4j(e))?;
            
        // This is a mock - we'll implement proper test infrastructure later
        // For now, we'll create a Graph with a placeholder client
        // The actual tests will be skipped until we have test infrastructure
        Err(SynapseError::Validation("Mock Graph - tests will be skipped until test DB is available".to_string()))
    }
}

/// Create a direct Neo4j connection (legacy function for backward compatibility)
pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Graph> {
    Graph::new_direct(uri, user, password).await
}

/// Internal function to create direct connection
async fn connect_direct(uri: &str, user: &str, password: &str) -> Result<Neo4jGraph> {
    // Build Neo4j configuration
    let config = ConfigBuilder::default()
        .uri(uri)
        .user(user)
        .password(password)
        .db("neo4j")
        .fetch_size(env::var("NEO4J_FETCH_SIZE").unwrap_or_else(|_| "500".to_string()).parse().unwrap_or(500))
        .max_connections(env::var("NEO4J_MAX_CONNECTIONS").unwrap_or_else(|_| "10".to_string()).parse().unwrap_or(10))
        .build()
        .map_err(|e| SynapseError::Neo4j(e))?;

    // Connect to Neo4j
    let client = Neo4jGraph::connect(config).await
        .map_err(|e| SynapseError::Neo4j(e))?;

    Ok(client)
}

pub async fn create_node(graph: &Graph, node: &Node) -> Result<()> {
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
    
    graph.provider.execute_query_void(
        neo4rs::query(query)
            .param("id", node.id.clone())
            .param("label", node.label.clone())
            .param("content", node.content.clone())
            .param("node_type", format!("{:?}", node.node_type))
            .param("tags", tags_json)
    ).await?;
    
    if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
        println!("Created/updated node: {} ({})", node.label, node.id);
    }
    
    Ok(())
}

pub async fn create_edge(graph: &Graph, edge: &Edge) -> Result<()> {
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
    
    graph.provider.execute_query_void(
        neo4rs::query(&query)
            .param("source_id", edge.source_id.clone())
            .param("target_id", edge.target_id.clone())
            .param("label", edge.label.clone())
            .param("edge_type", format!("{:?}", edge.edge_type))
    ).await?;
    
    if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
        println!("Created/updated edge: {} -> {} ({})", edge.source_id, edge.target_id, edge.label);
    }
    
    Ok(())
}

pub async fn query_nodes_by_type(graph: &Graph, node_type: &NodeType) -> Result<Vec<Node>> {
    let query = "
        MATCH (n { node_type: $node_type })
        RETURN n.id as id, n.label as label, n.content as content, 
               n.node_type as node_type, n.tags as tags
        ORDER BY n.label
    ";
    
    let node_type_clone = node_type.clone();
    graph.provider.execute_query_all(
        neo4rs::query(query)
            .param("node_type", format!("{:?}", node_type)),
        |row| {
            let id: String = row.get("id").unwrap_or_default();
            let label: String = row.get("label").unwrap_or_default();
            let content: String = row.get("content").unwrap_or_default();
            let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
            
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            
            let mut node = Node::new(node_type_clone.clone(), label, content);
            node.id = id;
            node.tags = tags;
            
            Ok(node)
        }
    ).await
}

pub async fn find_related_nodes(graph: &Graph, node_id: &str) -> Result<Vec<(Node, Edge)>> {
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
    
    let node_id_str = node_id.to_string();
    graph.provider.execute_query_all(
        neo4rs::query(query)
            .param("node_id", node_id),
        |row| {
            let id: String = row.get("id").unwrap_or_default();
            let label: String = row.get("label").unwrap_or_default();
            let content: String = row.get("content").unwrap_or_default();
            let node_type_str: String = row.get("node_type").unwrap_or_default();
            let tags_json: String = row.get("tags").unwrap_or_else(|_| "[]".to_string());
            let edge_label: String = row.get("edge_label").unwrap_or_default();
            let edge_type_str: String = row.get("edge_type").unwrap_or_default();
            
            // Parse node type
            let node_type = match node_type_str.as_str() {
                "File" => NodeType::File,
                "Rule" => NodeType::Rule,
                "Decision" => NodeType::Decision,
                "Function" => NodeType::Function,
                "Architecture" => NodeType::Architecture,
                "Component" => NodeType::Component,
                _ => NodeType::Rule, // Default fallback
            };
            
            // Parse edge type
            let edge_type = match edge_type_str.as_str() {
                "RelatesTo" => EdgeType::RelatesTo,
                "ImplementsRule" => EdgeType::ImplementsRule,
                "DefinedIn" => EdgeType::DefinedIn,
                "DependsOn" => EdgeType::DependsOn,
                "Contains" => EdgeType::Contains,
                "References" => EdgeType::References,
                _ => EdgeType::RelatesTo, // Default fallback
            };
            
            let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
            
            let mut node = Node::new(node_type, label, content);
            node.id = id.clone();
            node.tags = tags;
            
            let edge = Edge::new(node_id_str.clone(), id, edge_type, edge_label);
            
            Ok((node, edge))
        }
    ).await
}

pub async fn natural_language_query(graph: &Graph, query_text: &str) -> Result<String> {
    // Simple keyword-based search implementation
    let query_lower = query_text.to_lowercase();
    let keywords: Vec<&str> = query_lower.split_whitespace().collect();
    
    // Build a search query that looks for keywords in content, labels, and tags
    let cypher_query = "
        MATCH (n)
        WHERE ANY(keyword IN $keywords WHERE 
            toLower(n.label) CONTAINS toLower(keyword) OR 
            toLower(n.content) CONTAINS toLower(keyword) OR
            ANY(tag IN split(n.tags, ',') WHERE toLower(tag) CONTAINS toLower(keyword))
        )
        RETURN n.label as label, n.content as content, n.node_type as node_type
        ORDER BY n.label
        LIMIT 10
    ";
    
    let results = graph.provider.execute_query_all(
        neo4rs::query(cypher_query)
            .param("keywords", keywords),
        |row| {
            let label: String = row.get("label").unwrap_or_default();
            let content: String = row.get("content").unwrap_or_default();
            let node_type: String = row.get("node_type").unwrap_or_default();
            
            // Truncate content for display
            let content_preview = if content.len() > 100 {
                format!("{}...", &content[..97])
            } else {
                content
            };
            
            Ok(format!("- {} ({}): {}", label, node_type, content_preview))
        }
    ).await?;
    
    if results.is_empty() {
        Ok("No matching results found.".to_string())
    } else {
        Ok(format!("Found {} results:\n{}", results.len(), results.join("\n")))
    }
}

pub async fn batch_create(graph: &Graph, nodes: &[Node], edges: &[Edge]) -> Result<()> {
    // Validate all items first
    for node in nodes {
        node.validate()?;
    }
    for edge in edges {
        edge.validate()?;
    }
    
    // Create nodes first
    for node in nodes {
        create_node(graph, node).await?;
    }
    
    // Then create edges
    for edge in edges {
        create_edge(graph, edge).await?;
    }
    
    if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
        println!("Batch created {} nodes and {} edges", nodes.len(), edges.len());
    }
    
    Ok(())
}

pub async fn delete_node(graph: &Graph, node_id: &str) -> Result<()> {
    let query = "
        MATCH (n { id: $node_id })
        DETACH DELETE n
        RETURN count(n) as deleted_count
    ";
    
    let result = graph.provider.execute_query_single(
        neo4rs::query(query)
            .param("node_id", node_id),
        |row| {
            let deleted_count: i64 = row.get("deleted_count").unwrap_or(0);
            Ok(deleted_count)
        }
    ).await?;
    
    if let Some(deleted_count) = result {
        if deleted_count > 0 {
            if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                println!("Deleted node: {}", node_id);
            }
            Ok(())
        } else {
            Err(SynapseError::Validation(format!("Node not found: {}", node_id)))
        }
    } else {
        Err(SynapseError::Validation(format!("Node not found: {}", node_id)))
    }
}

pub async fn delete_edge(graph: &Graph, source_id: &str, target_id: &str) -> Result<()> {
    let query = "
        MATCH (source { id: $source_id })-[r]->(target { id: $target_id })
        DELETE r
        RETURN count(r) as deleted_count
    ";
    
    let result = graph.provider.execute_query_single(
        neo4rs::query(query)
            .param("source_id", source_id)
            .param("target_id", target_id),
        |row| {
            let deleted_count: i64 = row.get("deleted_count").unwrap_or(0);
            Ok(deleted_count)
        }
    ).await?;
    
    if let Some(deleted_count) = result {
        if deleted_count > 0 {
            if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
                println!("Deleted edge: {} -> {}", source_id, target_id);
            }
            Ok(())
        } else {
            Err(SynapseError::Validation(format!("Edge not found: {} -> {}", source_id, target_id)))
        }
    } else {
        Err(SynapseError::Validation(format!("Edge not found: {} -> {}", source_id, target_id)))
    }
}

pub async fn get_node_count(graph: &Graph) -> Result<i64> {
    let query = "MATCH (n) RETURN count(n) as count";
    
    let result = graph.provider.execute_query_single(
        neo4rs::query(query),
        |row| Ok(row.get("count").unwrap_or(0))
    ).await?;
    
    Ok(result.unwrap_or(0))
}

// Helper functions remain the same
fn _node_type_to_label(node_type: &NodeType) -> &'static str {
    match node_type {
        NodeType::File => "File",
        NodeType::Rule => "Rule", 
        NodeType::Decision => "Decision",
        NodeType::Function => "Function",
        NodeType::Architecture => "Architecture",
        NodeType::Component => "Component",
    }
}

fn edge_type_to_relationship(edge_type: &EdgeType) -> &'static str {
    match edge_type {
        EdgeType::RelatesTo => "RELATES_TO",
        EdgeType::ImplementsRule => "IMPLEMENTS_RULE",
        EdgeType::DefinedIn => "DEFINED_IN",
        EdgeType::DependsOn => "DEPENDS_ON",
        EdgeType::Contains => "CONTAINS",
        EdgeType::References => "REFERENCES",
        EdgeType::Inherits => "INHERITS",
        EdgeType::Overrides => "OVERRIDES",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_graph_operations() {
        // Skip test if Neo4j is not available
        if std::env::var("NEO4J_URI").is_err() {
            println!("Skipping graph test - NEO4J_URI not set");
            return;
        }
        
        let uri = std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
        let user = std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string());
        let password = std::env::var("NEO4J_PASSWORD").unwrap_or_else(|_| "password".to_string());
        
        let graph = match connect(&uri, &user, &password).await {
            Ok(g) => g,
            Err(_) => {
                println!("Skipping graph test - Neo4j connection failed");
                return;
            }
        };
        
        let node = Node::new(
            NodeType::Rule,
            "Test Rule".to_string(),
            "Test content".to_string(),
        );
        
        // Test creation
        assert!(create_node(&graph, &node).await.is_ok());
        
        // Test querying
        let nodes = query_nodes_by_type(&graph, &NodeType::Rule).await.unwrap();
        assert!(!nodes.is_empty());
        
        // Clean up test node
        let _ = delete_node(&graph, &node.id).await;
    }
}