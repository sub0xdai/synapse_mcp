# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Synapse MCP is a dynamic memory system for AI coding assistants that provides persistent project context through automated documentation indexing and knowledge graph querying. The system operates in two phases:

- **Write Phase**: Git commits trigger pre-commit hooks that parse changed Markdown files and update a Neo4j knowledge graph
- **Read Phase**: AI agents query via MCP server to retrieve structured project context in real-time

## Commands

### Build and Run
```bash
cargo build          # Build the project
cargo run            # Run the main binary
cargo check          # Fast syntax and type checking
```

### Testing and Quality
```bash
cargo test           # Run all tests
cargo clippy         # Rust linter for catching common mistakes
cargo fmt            # Format code according to Rust style guidelines
```

## Architecture

The codebase follows a modular Rust architecture with these key components:

### Core Data Models (src/main.rs)
- **Node**: Represents entities in the knowledge graph (Files, Rules, Decisions, Functions)
  - Contains: id, node_type, label, content, tags
- **Edge**: Represents relationships between nodes
  - Types: RelatesTo, ImplementsRule, DefinedIn
  - Contains: source_id, target_id, edge_type, label

### Technology Stack
- **Core Logic/Indexer**: Rust (performance-critical parsing)
- **Knowledge Graph**: Neo4j (connected data storage)
- **MCP Server**: Rust + Axum framework (high-performance API)
- **Hook Management**: pre-commit framework

### Data Flow
- **Memory Update**: Developer → git commit → pre-commit hook → Rust Indexer → Neo4j
- **AI Query**: AI Agent → MCP Server → Neo4j → Structured Response

## Development Context

The project is in early development with basic data structures defined. The main implementation focuses on:

1. Parsing Markdown documentation and YAML frontmatter
2. Extracting semantic relationships between project entities
3. Storing knowledge in Neo4j graph database
4. Providing real-time context via MCP server API

Performance target: Pre-commit hook indexing must complete under 500ms for average documentation changes.

## Document Format Requirements

### MCP Marker Requirement
Only markdown files with YAML frontmatter containing `mcp: synapse` will be processed. This allows multiple MCP servers to coexist without conflicts.

**Required frontmatter format:**
```yaml
---
mcp: synapse          # Required - marks document for Synapse MCP
type: rule            # Optional - node type (rule, decision, architecture, component, function)
title: "Document Title" # Optional - display name
tags: ["tag1", "tag2"] # Optional - categorization tags
# ... other metadata
---
```

**Supported node types:**
- `rule` - Development rules and guidelines
- `decision` - Architecture decisions and rationale  
- `architecture` - System architecture documentation
- `component` - Component specifications
- `function` - Function/method documentation
- Default: `file` (if no type specified)

**Document filtering:**
- Files without frontmatter: skipped
- Files with `mcp: other-server`: skipped  
- Files without `mcp` field: skipped
- Only `mcp: synapse` documents are indexed

Use `SYNAPSE_VERBOSE=1` with indexer for detailed filtering information.