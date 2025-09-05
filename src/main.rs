use clap::{Arg, Command};
use synapse_mcp::{graph, mcp_server, indexer};
use dotenv::dotenv;
use std::path::PathBuf;
use std::process;

mod cli;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    let matches = build_cli().get_matches();
    
    if let Err(e) = run_command(matches).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn build_cli() -> Command {
    Command::new("synapse")
        .version("0.2.0")
        .about("Synapse - AI Workspace Framework with Dynamic Memory")
        .long_about("A comprehensive framework for building AI-readable project documentation and context")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .about("Initialize a Synapse workspace")
                .long_about("Set up project scaffolding, templates, and automation hooks")
                .arg(
                    Arg::new("project-name")
                        .help("Project name for templates")
                        .required(false)
                        .index(1)
                )
                .arg(
                    Arg::new("template")
                        .short('t')
                        .long("template")
                        .help("Template type to use")
                        .value_parser(["rust", "python", "typescript", "generic"])
                        .default_value("generic")
                )
                .arg(
                    Arg::new("hooks")
                        .long("hooks")
                        .help("Install git hooks")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("index")
                .about("Index markdown files into knowledge graph")
                .arg(
                    Arg::new("files")
                        .help("Markdown files to index")
                        .required(true)
                        .num_args(1..)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Parse files but don't update database")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("parallel")
                        .short('j')
                        .long("parallel")
                        .help("Number of parallel workers")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("4")
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Verbose output")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("context")
                .about("Generate AI context from knowledge graph")
                .arg(
                    Arg::new("scope")
                        .short('s')
                        .long("scope")
                        .help("Context scope")
                        .value_parser(["all", "rules", "architecture", "decisions", "test", "api"])
                        .default_value("all")
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format")
                        .value_parser(["markdown", "json", "plain"])
                        .default_value("markdown")
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output file")
                        .default_value(".synapse_context")
                )
                .arg(
                    Arg::new("filter")
                        .long("filter")
                        .help("Filter by file pattern or tags")
                        .num_args(0..)
                )
        )
        .subcommand(
            Command::new("query")
                .about("Query knowledge graph directly")
                .arg(
                    Arg::new("query")
                        .help("Natural language query")
                        .required(true)
                        .index(1)
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format")
                        .value_parser(["markdown", "json", "plain"])
                        .default_value("markdown")
                )
        )
        .subcommand(
            Command::new("serve")
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
                    Arg::new("host")
                        .long("host")
                        .help("Host to bind to")
                        .default_value("localhost")
                )
        )
        .subcommand(
            Command::new("check")
                .about("Check files against synapse rules (Write Hook)")
                .long_about("Enforces FORBIDDEN and REQUIRED rules against specified files. Used by pre-commit hooks.")
                .arg(
                    Arg::new("files")
                        .help("Files to check against rules")
                        .required(true)
                        .num_args(1..)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed checking information")
                        .action(clap::ArgAction::SetTrue)
                )
                .arg(
                    Arg::new("dry-run")
                        .long("dry-run")
                        .help("Parse and check files but don't enforce (exit 0)")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("enforce-context")
                .about("Generate rule context for AI assistant (Read Hook)")
                .long_about("Provides structured rule information for a file path to guide AI development.")
                .arg(
                    Arg::new("path")
                        .help("File path to get rules for")
                        .required(true)
                        .index(1)
                        .value_parser(clap::value_parser!(PathBuf))
                )
                .arg(
                    Arg::new("format")
                        .short('f')
                        .long("format")
                        .help("Output format")
                        .value_parser(["markdown", "json", "plain"])
                        .default_value("markdown")
                )
                .arg(
                    Arg::new("output")
                        .short('o')
                        .long("output")
                        .help("Output to file instead of stdout")
                        .value_parser(clap::value_parser!(String))
                )
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed context generation information")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("status")
                .about("Check system status and health")
                .arg(
                    Arg::new("verbose")
                        .short('v')
                        .long("verbose")
                        .help("Show detailed status")
                        .action(clap::ArgAction::SetTrue)
                )
        )
        .subcommand(
            Command::new("demo")
                .about("Run system demonstration")
                .hide(true)
        )
        .arg(
            Arg::new("neo4j-uri")
                .long("neo4j-uri")
                .help("Neo4j database URI")
                .global(true)
                .default_value("bolt://localhost:7687")
        )
        .arg(
            Arg::new("neo4j-user")
                .long("neo4j-user")
                .help("Neo4j username")
                .global(true)
                .default_value("neo4j")
        )
        .arg(
            Arg::new("neo4j-password")
                .long("neo4j-password")
                .help("Neo4j password")
                .global(true)
                .default_value("password")
        )
}

async fn run_command(matches: clap::ArgMatches) -> anyhow::Result<()> {
    let neo4j_uri = matches.get_one::<String>("neo4j-uri").unwrap();
    let neo4j_user = matches.get_one::<String>("neo4j-user").unwrap();
    let neo4j_password = matches.get_one::<String>("neo4j-password").unwrap();

    match matches.subcommand() {
        Some(("init", sub_matches)) => {
            cli::commands::init::handle_init(sub_matches).await?
        }
        Some(("index", sub_matches)) => {
            cli::commands::index::handle_index(sub_matches, neo4j_uri, neo4j_user, neo4j_password).await?
        }
        Some(("context", sub_matches)) => {
            cli::commands::context::handle_context(sub_matches, neo4j_uri, neo4j_user, neo4j_password).await?
        }
        Some(("query", sub_matches)) => {
            cli::commands::query::handle_query(sub_matches, neo4j_uri, neo4j_user, neo4j_password).await?
        }
        Some(("serve", sub_matches)) => {
            let port = *sub_matches.get_one::<u16>("port").unwrap();
            let host = sub_matches.get_one::<String>("host").unwrap();
            
            println!("🚀 Starting Synapse MCP server on {}:{}", host, port);
            println!("📊 Connecting to Neo4j at {}", neo4j_uri);
            
            match graph::connect(neo4j_uri, neo4j_user, neo4j_password).await {
                Ok(graph_conn) => {
                    mcp_server::start_server(graph_conn, port).await?
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Failed to connect to Neo4j: {}", e));
                }
            }
        }
        Some(("check", sub_matches)) => {
            cli::commands::check::handle_check(sub_matches).await?
        }
        Some(("enforce-context", sub_matches)) => {
            cli::commands::enforce_context::handle_enforce_context(sub_matches).await?
        }
        Some(("status", sub_matches)) => {
            cli::commands::status::handle_status(sub_matches, neo4j_uri, neo4j_user, neo4j_password).await?
        }
        Some(("demo", _)) => {
            run_demo().await;
        }
        _ => {
            unreachable!("Command parsing should ensure we never reach this");
        }
    }
    
    Ok(())
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

