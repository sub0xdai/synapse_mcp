// Neo4j graph database operations
use crate::{Node, Edge, NodeType, EdgeType, Result, SynapseError};
use neo4rs::{Graph as Neo4jGraph, ConfigBuilder};
use std::env;

pub struct Graph {
    client: Neo4jGraph,
}

pub async fn connect(uri: &str, user: &str, password: &str) -> Result<Graph> {
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

    Ok(Graph { client })
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
    
    let mut result = graph.client.execute(
        neo4rs::query(query)
            .param("id", node.id.clone())
            .param("label", node.label.clone())
            .param("content", node.content.clone())
            .param("node_type", format!("{:?}", node.node_type))
            .param("tags", tags_json)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
        println!("Created/updated node: {} ({})", node.label, node.id);
    }
    
    result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
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
    
    let mut result = graph.client.execute(
        neo4rs::query(&query)
            .param("source_id", edge.source_id.clone())
            .param("target_id", edge.target_id.clone())
            .param("label", edge.label.clone())
            .param("edge_type", format!("{:?}", edge.edge_type))
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if env::var("SYNAPSE_VERBOSE").unwrap_or_else(|_| "false".to_string()) == "true" {
        println!("Created/updated edge: {} -> {} ({})", edge.source_id, edge.target_id, edge.label);
    }
    
    result.next().await.map_err(|e| SynapseError::Neo4j(e))?;
    Ok(())
}

pub async fn query_nodes_by_type(graph: &Graph, node_type: &NodeType) -> Result<Vec<Node>> {
    let query = "
        MATCH (n { node_type: $node_type })
        RETURN n.id as id, n.label as label, n.content as content, 
               n.node_type as node_type, n.tags as tags
        ORDER BY n.label
    ";
    
    let mut result = graph.client.execute(
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
    
    Ok(nodes)
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
    
    let mut result = graph.client.execute(
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
        
        let edge = Edge::new(node_id.to_string(), id, edge_type, edge_label);
        
        relationships.push((node, edge));
    }
    
    Ok(relationships)
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
    
    let mut result = graph.client.execute(
        neo4rs::query(cypher_query)
            .param("keywords", keywords)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    let mut results = Vec::new();
    let mut count = 0;
    
    while let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
        let label: String = row.get("label").unwrap_or_default();
        let content: String = row.get("content").unwrap_or_default();
        let node_type: String = row.get("node_type").unwrap_or_default();
        
        // Truncate content for display
        let content_preview = if content.len() > 100 {
            format!("{}...", &content[..97])
        } else {
            content
        };
        
        results.push(format!("- {} ({}): {}", label, node_type, content_preview));
        count += 1;
    }
    
    if results.is_empty() {
        Ok("No matching results found.".to_string())
    } else {
        Ok(format!("Found {} results:\n{}", count, results.join("\n")))
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
    
    let mut result = graph.client.execute(
        neo4rs::query(query)
            .param("node_id", node_id)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
        let deleted_count: i64 = row.get("deleted_count").unwrap_or(0);
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
    
    let mut result = graph.client.execute(
        neo4rs::query(query)
            .param("source_id", source_id)
            .param("target_id", target_id)
    ).await.map_err(|e| SynapseError::Neo4j(e))?;
    
    if let Some(row) = result.next().await.map_err(|e| SynapseError::Neo4j(e))? {
        let deleted_count: i64 = row.get("deleted_count").unwrap_or(0);
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