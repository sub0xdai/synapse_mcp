---
mcp: synapse
type: architecture
title: "Synapse MCP Architecture Decision"
tags: ["architecture", "neo4j", "rust", "mcp"]
---

# Architecture Decision ADR-001

## Context
Need a dynamic memory system for AI coding assistants that provides persistent project context.

## Decision
Use Neo4j graph database with Rust backend for optimal performance and relationship queries.

## Status
Implemented ✅

## Consequences
- Enables complex relationship traversal
- Provides sub-500ms query performance
- Supports incremental updates via git hooks

## Components
- **Neo4j Database**: Stores knowledge graph
- **Rust Indexer**: Parses markdown files
- **MCP Server**: Provides API interface
- **Git Hooks**: Automatic indexing on commits

## References
- Implements [Performance Rule PR-001] requirements
- Related to [Component Database] implementation