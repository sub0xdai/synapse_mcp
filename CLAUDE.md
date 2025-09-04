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

## Automation & Hooks

Synapse MCP provides full automation through dual-hook system for seamless AI memory integration.

### Setup
```bash
# One-time setup of all automation hooks
./setup-hooks.sh

# Manual setup steps (if needed):
uv tool install pre-commit    # Install pre-commit framework
pre-commit install            # Install git hooks
```

### Git Pre-Commit Hook (Write Path)
Automatically indexes markdown files with `mcp: synapse` marker on every commit:

```bash
# Automatic - runs on git commit
git add docs/new-rule.md
git commit -m "Add new rule"     # Triggers indexing automatically

# Manual testing
pre-commit run --all-files       # Test hooks on all files
pre-commit run --files file.md   # Test specific file
```

### Claude Context Hook (Read Path)
Automatically provides project context to AI agents:

```bash
# Generate context for Claude
./claude-hook.sh context

# Check if MCP server is running
./claude-hook.sh status

# Start/stop server manually
./claude-hook.sh start
./claude-hook.sh stop
```

### Context Integration
The context hook generates `.synapse_context` with current project rules, architecture decisions, and relevant context:

```markdown
# Example generated context
# SYNAPSE MCP CONTEXT
# Auto-generated project context from knowledge graph

# Project Rules
- **Performance Rule PR-001**: All async operations must complete within 500ms
- **Testing Rule TR-001**: All public APIs must have integration tests

# Architecture Decisions  
- **Technology Stack Decision**: Neo4j + Rust for optimal performance
- **API Design Decision**: REST API with JSON responses
```

### Environment Variables
```bash
SYNAPSE_MCP_URL=http://localhost:8080    # MCP server URL
SYNAPSE_CONTEXT_FILE=.synapse_context    # Context file location
SYNAPSE_VERBOSE=true                     # Enable verbose logging
```

### Performance
- Pre-commit indexing: <500ms for typical markdown files
- Context generation: <200ms for full project context
- MCP server queries: <100ms average response time