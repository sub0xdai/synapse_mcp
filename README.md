```markdown
# Synapse MCP

A dynamic memory system for AI coding assistants that provides persistent project context through automated documentation indexing and knowledge graph querying.

## How It Works

Synapse operates in two phases:

**Write Phase** (Memory Updates)
- Git commit triggers pre-commit hook
- Indexer parses changed Markdown files
- Knowledge graph updates with new relationships

**Read Phase** (AI Context)
- AI agents query via MCP server
- Natural language queries convert to Cypher
- Structured project context returned in real-time

## Tech Stack

- **Core Indexer**: Rust (performance-critical parsing)
- **Knowledge Graph**: Neo4j (connected data storage)
- **MCP Server**: Rust + Axum (high-performance API)
- **Hook Management**: pre-commit framework

## Key Components

- **Git Hook**: Detects documentation changes on commit
- **Rust Indexer**: Extracts nodes and relationships from Markdown
- **Neo4j Database**: Stores project rules, decisions, and architecture
- **MCP Server**: Translates natural language to graph queries

## Data Flow

**Memory Update**: `Developer → git commit → pre-commit → Indexer → Neo4j`

**AI Query**: `AI Agent → MCP Server → Neo4j → Structured Response`

## Benefits

- Automatic documentation indexing on every commit
- Complex relationship understanding through knowledge graphs
- Real-time context provision for AI coding assistants
- Persistent memory of project architecture and rules
```
