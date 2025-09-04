# Synapse MCP

A dynamic memory system for AI coding assistants that provides persistent project context through automated documentation indexing and knowledge graph querying.

## Status

**Current Version**: 0.1.0  
**Neo4j Integration**: Complete and tested  
**Test Coverage**: 36 tests passing  
**Database**: Neo4j graph database with real operations

## Quick Start

### Prerequisites
- Rust 1.70+
- Neo4j database (local or remote)
- Environment variables configured

### Setup
1. Clone and configure environment:
   ```bash
   git clone <repository>
   cd synapse_mcp
   cp .env.example .env
   # Edit .env with your Neo4j credentials
   ```

2. Test connection:
   ```bash
   cargo run --bin test_connection
   ```

3. Index markdown documentation:
   ```bash
   cargo run --bin indexer docs/*.md
   ```

4. Start MCP server:
   ```bash
   cargo run server --port 8080
   ```

## How It Works

Synapse operates in two phases:

**Write Phase** (Memory Updates)
- Indexer parses Markdown files with `mcp: synapse` frontmatter
- Knowledge graph stores nodes (rules, decisions, architecture) and relationships
- Real-time updates to Neo4j database with UPSERT operations

**Read Phase** (AI Context)
- AI agents query via REST API endpoints
- Natural language queries search across content, labels, and tags
- Structured project context returned with related nodes and edges

## Architecture

### Core Components
- **Rust Indexer**: High-performance markdown parsing with YAML frontmatter validation
- **Neo4j Database**: Graph storage with real Cypher operations (no stubs)
- **MCP Server**: REST API built with Axum for AI agent integration
- **Data Models**: Strongly-typed Node and Edge structures with validation

### Database Schema
**Node Types**: File, Rule, Decision, Function, Architecture, Component  
**Relationship Types**: RelatesTo, ImplementsRule, DefinedIn, DependsOn, Contains, References  
**Properties**: Labels, content, tags, metadata, timestamps

### Data Flow
**Memory Update**: `Markdown Files → Indexer → Validation → Neo4j Graph`  
**AI Query**: `Natural Language → Keyword Search → Cypher Query → Structured Results`

## API Endpoints

- `GET /health` - Server health check
- `POST /query` - Natural language knowledge graph queries
  ```json
  {
    "query": "find rules about performance"
  }
  ```

## Development

### Run Tests
```bash
cargo test                    # Full test suite
cargo test --lib             # Unit tests only
```

### Performance
- Indexing target: <500ms per batch
- Query response: <100ms typical
- Supports concurrent operations

### Environment Variables
```bash
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password
NEO4J_DATABASE=neo4j
NEO4J_MAX_CONNECTIONS=10
NEO4J_FETCH_SIZE=500
SYNAPSE_VERBOSE=true
```

## Document Format

Markdown files must include YAML frontmatter with `mcp: synapse`:

```yaml
---
mcp: synapse
type: rule
title: "Performance Guidelines"
tags: ["performance", "guidelines"]
---

# Performance Guidelines

Content here will be indexed into the knowledge graph...
```

## Benefits

- **Automatic Documentation Indexing**: Parse and store project knowledge
- **Complex Relationship Queries**: Traverse connected concepts and dependencies  
- **Real-time AI Context**: Instant access to project rules and decisions
- **Persistent Memory**: Maintain architecture knowledge across development sessions
- **Multi-MCP Support**: Only processes documents marked with `mcp: synapse`
