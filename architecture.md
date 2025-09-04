```markdown
# Architecture: Synapse MCP

## 1. Core Concept

Synapse MCP operates in two distinct phases:
- **Write Phase:** Triggered by git commit to update its memory
- **Read Phase:** Via an MCP server to provide context to an AI

## 2. Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Core Logic/Indexer | Rust | Performance-critical parsing and indexing. |
| Knowledge Graph | Neo4j | Storing and querying connected data. |
| MCP Server | Rust (Axum Framework) | Lightweight, high-performance API. |
| Hook Management | pre-commit Framework | User-friendly git hook installation. |

## 3. Component Breakdown

### Git Hook (pre-commit)
- **Trigger:** Runs on git commit
- **Action:** Detects changed `.md` files and executes the Rust indexer binary, passing the file paths as arguments

### Indexer (Rust Core)
- **Input:** A list of changed Markdown file paths
- **Action:** Parses each file's content and YAML frontmatter to extract nodes (Rules, Decisions, Files) and edges (Relationships)
- **Output:** Cypher queries that create, update, or delete data in the Neo4j database

### Knowledge Graph (Neo4j)
- **Schema:** Comprised of Node labels (e.g., Rule, Decision, File) and Edge types (e.g., RELATES_TO, DEFINED_IN)
- **Function:** Serves as the persistent, queryable "brain"

### MCP Server (Rust/Axum)
- **Function:** Exposes a single API endpoint (e.g., `POST /query`)
- **Action:** Receives a natural language query, translates it into a Cypher query against Neo4j, and returns a structured JSON response to the AI agent

## 4. Data Flow Diagram

**Write Path (Memory Update):**
```
Developer → git commit → pre-commit hook → Rust Indexer → Neo4j DB
```

**Read Path (AI Query):**
```
AI Agent → Serena MCP → Synapse MCP Server → Neo4j DB → AI Agent
```
