use clap::{Arg, Command};
use synapse_mcp::{graph, mcp_server};

#[tokio::main]
async fn main() {
    let matches = Command::new("synapse-mcp")
        .version("0.1.0")
        .about("Synapse MCP - Dynamic memory system for AI coding assistants")
        .subcommand(
            Command::new("server")
                .about("Start the MCP server")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("Port to listen on")
                        .default_value("8080")
                        .value_parser(clap::value_parser!(u16))
                )
                .arg(
                    Arg::new("neo4j-uri")
                        .long("neo4j-uri")
                        .help("Neo4j database URI")
                        .default_value("bolt://localhost:7687")
                )
                .arg(
                    Arg::new("neo4j-user")
                        .long("neo4j-user")
                        .help("Neo4j username")
                        .default_value("neo4j")
                )
                .arg(
                    Arg::new("neo4j-password")
                        .long("neo4j-password")
                        .help("Neo4j password")
                        .default_value("password")
                )
        )
        .subcommand(
            Command::new("demo")
                .about("Run a demonstration of the system")
        )
        .get_matches();

    match matches.subcommand() {
        Some(("server", sub_matches)) => {
            let port = *sub_matches.get_one::<u16>("port").unwrap();
            let neo4j_uri = sub_matches.get_one::<String>("neo4j-uri").unwrap();
            let neo4j_user = sub_matches.get_one::<String>("neo4j-user").unwrap();
            let neo4j_password = sub_matches.get_one::<String>("neo4j-password").unwrap();

            println!("Connecting to Neo4j at {}", neo4j_uri);
            
            match graph::connect(neo4j_uri, neo4j_user, neo4j_password).await {
                Ok(graph_conn) => {
                    if let Err(e) = mcp_server::start_server(graph_conn, port).await {
                        eprintln!("Server error: {}", e);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to Neo4j: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(("demo", _)) => {
            run_demo().await;
        }
        _ => {
            println!("Synapse MCP v0.1.0");
            println!("A dynamic memory system for AI coding assistants");
            println!();
            println!("Use --help to see available commands");
            println!();
            println!("Quick start:");
            println!("  synapse-mcp demo                    # Run a demonstration");
            println!("  synapse-mcp server --port 8080      # Start MCP server");
            println!("  synapse-indexer file1.md file2.md   # Index markdown files");
        }
    }
}

async fn run_demo() {
    println!("🧠 Synapse MCP Demonstration");
    println!("============================");
    println!();

    // Connect to graph (using stub implementation)
    println!("1. Connecting to knowledge graph...");
    let graph = match graph::connect("demo://", "demo", "demo").await {
        Ok(g) => {
            println!("   ✓ Connected successfully");
            g
        }
        Err(e) => {
            println!("   ✗ Failed to connect: {}", e);
            return;
        }
    };

    println!();
    println!("2. Testing natural language queries...");
    
    let queries = vec![
        "Find all rules about performance",
        "Show me architecture decisions",
        "What are the coding standards?",
    ];

    for query in queries {
        println!("   Query: \"{}\"", query);
        match graph::natural_language_query(&graph, query).await {
            Ok(result) => {
                println!("   Result: {}", result.lines().next().unwrap_or("No response"));
            }
            Err(e) => {
                println!("   Error: {}", e);
            }
        }
        println!();
    }

    println!("3. System architecture overview:");
    println!("   📁 Data Models: Node and Edge structures for knowledge representation");
    println!("   📄 Indexer: Parses markdown files with YAML frontmatter");
    println!("   🗄️  Graph Database: Stores nodes and relationships (Neo4j ready)");
    println!("   🌐 MCP Server: Provides API for AI agents to query knowledge");
    println!("   🔗 Git Hook: Automatically indexes changes on commit");
    println!();

    println!("4. Performance characteristics:");
    println!("   • Target indexing speed: <500ms per batch");
    println!("   • Query response time: <100ms typical");
    println!("   • Supports complex relationship queries");
    println!("   • Incremental updates via git hooks");
    println!();

    println!("✨ Demo complete! System is ready for integration.");
    println!();
    println!("Next steps:");
    println!("  • Set up Neo4j database for production use");
    println!("  • Install git hooks in your repository");
    println!("  • Start the MCP server: synapse-mcp server");
    println!("  • Connect your AI coding assistant via MCP protocol");
}

