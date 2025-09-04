use synapse_mcp::{Node, Edge, NodeType, EdgeType};

// Note: These are integration tests that require a running Neo4j instance
// For CI/CD, consider using testcontainers-rs to spin up Neo4j automatically

#[tokio::test]
async fn test_graph_connection() {
    // Skip if Neo4j is not available in test environment
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let uri = std::env::var("NEO4J_TEST_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_TEST_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_TEST_PASSWORD").unwrap_or_else(|_| "password".to_string());
    
    let result = synapse_mcp::graph::connect(&uri, &user, &password).await;
    assert!(result.is_ok(), "Failed to connect to Neo4j: {:?}", result.err());
}

#[tokio::test]
async fn test_create_node() {
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let graph = create_test_graph().await;
    
    let node = Node::new(
        NodeType::Rule,
        "Test Rule".to_string(),
        "This is a test rule".to_string(),
    );
    
    let result = synapse_mcp::graph::create_node(&graph, &node).await;
    assert!(result.is_ok());
    
    // Clean up
    let _ = synapse_mcp::graph::delete_node(&graph, &node.id).await;
}

#[tokio::test]
async fn test_create_edge() {
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let graph = create_test_graph().await;
    
    // Create two nodes first
    let node1 = Node::new(NodeType::Rule, "Rule 1".to_string(), "Content 1".to_string());
    let node2 = Node::new(NodeType::Decision, "Decision 1".to_string(), "Content 2".to_string());
    
    let _ = synapse_mcp::graph::create_node(&graph, &node1).await.unwrap();
    let _ = synapse_mcp::graph::create_node(&graph, &node2).await.unwrap();
    
    let edge = Edge::new(
        node1.id.clone(),
        node2.id.clone(),
        EdgeType::RelatesTo,
        "relates to".to_string(),
    );
    
    let result = synapse_mcp::graph::create_edge(&graph, &edge).await;
    assert!(result.is_ok());
    
    // Clean up
    let _ = synapse_mcp::graph::delete_edge(&graph, &edge.source_id, &edge.target_id).await;
    let _ = synapse_mcp::graph::delete_node(&graph, &node1.id).await;
    let _ = synapse_mcp::graph::delete_node(&graph, &node2.id).await;
}

#[tokio::test]
async fn test_query_nodes_by_type() {
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let graph = create_test_graph().await;
    
    let rule_node = Node::new(
        NodeType::Rule,
        "Query Test Rule".to_string(),
        "Test content".to_string(),
    );
    
    let _ = synapse_mcp::graph::create_node(&graph, &rule_node).await.unwrap();
    
    let result = synapse_mcp::graph::query_nodes_by_type(&graph, &NodeType::Rule).await;
    assert!(result.is_ok());
    
    let nodes = result.unwrap();
    assert!(!nodes.is_empty());
    assert!(nodes.iter().any(|n| n.id == rule_node.id));
    
    // Clean up
    let _ = synapse_mcp::graph::delete_node(&graph, &rule_node.id).await;
}

#[tokio::test]
async fn test_query_relationships() {
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let graph = create_test_graph().await;
    
    let rule = Node::new(NodeType::Rule, "Rule".to_string(), "Rule content".to_string());
    let impl_node = Node::new(NodeType::Function, "Function".to_string(), "Function content".to_string());
    
    let _ = synapse_mcp::graph::create_node(&graph, &rule).await.unwrap();
    let _ = synapse_mcp::graph::create_node(&graph, &impl_node).await.unwrap();
    
    let edge = Edge::new(
        impl_node.id.clone(),
        rule.id.clone(),
        EdgeType::ImplementsRule,
        "implements".to_string(),
    );
    
    let _ = synapse_mcp::graph::create_edge(&graph, &edge).await.unwrap();
    
    let result = synapse_mcp::graph::find_related_nodes(&graph, &rule.id).await;
    assert!(result.is_ok());
    
    let related = result.unwrap();
    assert!(!related.is_empty());
    assert!(related.iter().any(|(n, _)| n.id == impl_node.id));
    
    // Clean up
    let _ = synapse_mcp::graph::delete_edge(&graph, &edge.source_id, &edge.target_id).await;
    let _ = synapse_mcp::graph::delete_node(&graph, &rule.id).await;
    let _ = synapse_mcp::graph::delete_node(&graph, &impl_node.id).await;
}

#[tokio::test]
async fn test_natural_language_query() {
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let graph = create_test_graph().await;
    
    // Create test data
    let performance_rule = Node::new(
        NodeType::Rule,
        "Performance Rule".to_string(),
        "Use Rust for performance-critical code".to_string(),
    ).with_tags(vec!["performance".to_string(), "rust".to_string()]);
    
    let _ = synapse_mcp::graph::create_node(&graph, &performance_rule).await.unwrap();
    
    // Test natural language query
    let result = synapse_mcp::graph::natural_language_query(
        &graph, 
        "Find all rules about performance"
    ).await;
    
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(response.contains("performance") || response.contains("Performance"));
    
    // Clean up
    let _ = synapse_mcp::graph::delete_node(&graph, &performance_rule.id).await;
}

#[tokio::test] 
async fn test_batch_operations() {
    if std::env::var("NEO4J_TEST_URI").is_err() {
        println!("Skipping Neo4j tests - NEO4J_TEST_URI not set");
        return;
    }
    
    let graph = create_test_graph().await;
    
    let nodes = vec![
        Node::new(NodeType::Rule, "Batch Rule 1".to_string(), "Content 1".to_string()),
        Node::new(NodeType::Rule, "Batch Rule 2".to_string(), "Content 2".to_string()),
        Node::new(NodeType::Decision, "Batch Decision".to_string(), "Decision content".to_string()),
    ];
    
    let edges = vec![
        Edge::new(
            nodes[0].id.clone(),
            nodes[2].id.clone(),
            EdgeType::RelatesTo,
            "relates to decision".to_string(),
        ),
    ];
    
    let result = synapse_mcp::graph::batch_create(&graph, &nodes, &edges).await;
    assert!(result.is_ok());
    
    // Verify nodes were created
    let rule_nodes = synapse_mcp::graph::query_nodes_by_type(&graph, &NodeType::Rule).await.unwrap();
    assert!(rule_nodes.len() >= 2);
    
    // Clean up
    for node in &nodes {
        let _ = synapse_mcp::graph::delete_node(&graph, &node.id).await;
    }
}

async fn create_test_graph() -> synapse_mcp::graph::Graph {
    let uri = std::env::var("NEO4J_TEST_URI").unwrap_or_else(|_| "bolt://localhost:7687".to_string());
    let user = std::env::var("NEO4J_TEST_USER").unwrap_or_else(|_| "neo4j".to_string());
    let password = std::env::var("NEO4J_TEST_PASSWORD").unwrap_or_else(|_| "password".to_string());
    
    synapse_mcp::graph::connect(&uri, &user, &password)
        .await
        .expect("Failed to connect to test Neo4j instance")
}