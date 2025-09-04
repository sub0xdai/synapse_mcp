use anyhow::Result;
use clap::ArgMatches;

use synapse_mcp::graph;

pub async fn handle_query(
    matches: &ArgMatches,
    neo4j_uri: &str,
    neo4j_user: &str,
    neo4j_password: &str,
) -> Result<()> {
    let query = matches.get_one::<String>("query").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    
    println!("🔍 Querying: \"{}\"", query);
    
    // Connect to Neo4j
    let graph_conn = graph::connect(neo4j_uri, neo4j_user, neo4j_password).await?;
    
    // Execute natural language query
    let result = graph::natural_language_query(&graph_conn, query).await?;
    
    // Format and display results
    match format.as_str() {
        "json" => {
            // Convert to JSON format
            let json_result = serde_json::json!({
                "query": query,
                "result": result,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            println!("{}", serde_json::to_string_pretty(&json_result)?);
        }
        "plain" => {
            println!("Query: {}", query);
            println!("Result: {}", result);
        }
        "markdown" | _ => {
            println!("# Query Results\n");
            println!("**Query**: {}\n", query);
            println!("**Result**:\n");
            println!("{}", result);
        }
    }
    
    Ok(())
}