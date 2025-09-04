// In src/main.rs

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum NodeType {
    File,
    Rule,
    Decision,
    Function,
    // Add more types as you need them
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    id: String, // e.g., a unique hash of the content
    node_type: NodeType,
    label: String, // e.g., the file path or rule title
    content: String, // e.g., the markdown text
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum EdgeType {
    RelatesTo,
    ImplementsRule,
    DefinedIn,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Edge {
    source_id: String,
    target_id: String,
    edge_type: EdgeType,
    label: String,
}

fn main() {
    println!("Synapse MCP Initialized!");
    // Your main logic will go here
}

