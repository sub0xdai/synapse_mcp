# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This project is guided by the **Feinburhm Constant ($\mathcal{F}_c$)**, a principle representing the relentless pursuit of a single, unified Gödel number that unites all mentor vernaculars. Every architectural decision and workflow must align with this constant.

To that end, Synapse MCP is a dynamic memory system designed to imbue AI coding assistants with this project's core principles. It provides persistent context through automated documentation indexing, rule enforcement, and knowledge graph querying. The system operates through a dual-hook architecture to ensure all development remains on a monotonic and constructive path:

- **Write Phase**: Git commits trigger pre-commit hooks that validate files against project rules and index markdown documentation. This ensures history is immutable and compliant.
- **Read Phase**: AI agents query the MCP server to retrieve structured project context and applicable rules in real-time, ensuring all generated code adheres to the project's philosophy from the start.

## Commands

### Build and Development
```bash
cargo build                    # Build the project
cargo build --release          # Optimized release build
cargo run                      # Run the main binary with subcommands
cargo check                    # Fast syntax and type checking
cargo run -- --help            # Show all available CLI commands
```

### Testing and Quality
```bash
cargo test                     # Run all tests
cargo test --lib               # Run library tests only
cargo test integration_        # Run integration tests
cargo clippy                   # Rust linter for catching common mistakes
cargo fmt                      # Format code according to Rust style guidelines
```

### Rule Enforcement
```bash
cargo run -- check src/*.rs              # Check files against rules
cargo run -- check . --verbose           # Check with detailed output
cargo run -- enforce-context src/main.rs # Generate AI context for file
```

### Server Operations
```bash
cargo run -- serve                       # Start MCP server on localhost:8080
cargo run -- serve --enable-enforcer     # Start with rule enforcement endpoints
cargo run -- status                      # Check system and database health
```

### Health Check Endpoints
```bash
curl http://localhost:8080/health         # Simple health check (returns "OK")
curl http://localhost:8080/status         # Detailed health status (JSON)
```

### Benchmarking
```bash
cargo bench                    # Run performance benchmarks
```

## Architecture

The codebase follows a modular Rust architecture with these key components:

### Core Data Models (src/models.rs)
- **Node**: Represents entities in the knowledge graph (Files, Rules, Decisions, Functions, Components)
  - Contains: id, node_type, label, content, tags, metadata
  - Types: File, Rule, Decision, Function, Architecture, Component
- **Edge**: Represents relationships between nodes
  - Types: RelatesTo, ImplementsRule, DefinedIn
  - Contains: source_id, target_id, edge_type, label

### Rule Enforcement System
- **PatternEnforcer** (src/enforcement.rs): Core rule validation and context generation engine
- **RuleGraph** (src/rule_graph.rs): In-memory graph of rule relationships with directory-based inheritance
- **Rule Discovery** (src/rules/): Recursive scanning and parsing of `.synapse/*.md` rule files
- **Rule Types**: FORBIDDEN, REQUIRED, STANDARD, CONVENTION patterns

### Technology Stack
- **Core Logic**: Rust (performance-critical parsing and enforcement)
- **Knowledge Graph**: Neo4j (connected data storage and complex queries)
- **MCP Server**: Rust + Axum framework (high-performance API with rule enforcement endpoints)
- **Hook Management**: pre-commit framework (automated git integration)
- **CLI Interface**: clap with comprehensive subcommands

### Key Source Files
- `src/main.rs`: CLI entry point and command routing (490 lines)
- `src/models.rs`: Core data structures and graph entities (19k lines)
- `src/graph.rs`: Neo4j integration and graph operations (16k lines)
- `src/mcp_server.rs`: HTTP server and MCP protocol implementation (31k lines)
- `src/enforcement.rs`: Rule validation and pattern matching (11k lines)
- `src/rule_graph.rs`: Rule inheritance and relationship management (12k lines)
- `src/config.rs`: Configuration management and environment setup (14k lines)
- `src/health.rs`: Health monitoring and dependency checking system

### Data Flow
- **Write Hook**: Developer → git commit → pre-commit → PatternEnforcer validation → rule indexing → Neo4j
- **Read Hook**: AI Agent → MCP Server → RuleGraph → context generation → structured response
- **Knowledge Graph**: Markdown files → indexer → Neo4j → natural language queries

## Development Context

The project implements a comprehensive rule enforcement and context generation system. Key implementation areas:

1. **Rule System**: Directory-based inheritance with `.synapse/*.md` files containing YAML frontmatter
2. **Pattern Matching**: Regex-based enforcement of FORBIDDEN/REQUIRED patterns in source files
3. **Context Generation**: Multi-format output (Markdown, JSON, Plain) for AI assistant integration
4. **Performance**: Sub-500ms rule validation, <100ms context generation
5. **Neo4j Integration**: Full CRUD operations with relationship querying
6. **MCP Protocol**: HTTP-based server with health checks and enforcement endpoints
7. **Health Monitoring**: Comprehensive health checks with JSON status reporting
8. **Structured Logging**: Enhanced JSON logging with tracing instrumentation

Performance targets: Pre-commit hook validation <500ms, context generation <200ms, MCP queries <100ms.

## Rule System

### Rule File Format
Rules are defined in `.md` files within `.synapse/` directories using YAML frontmatter:

```yaml
---
mcp: synapse                              # Required - marks file for Synapse MCP
type: rule                                # Optional - node type
inherits: ["../.synapse/security.md"]     # Optional - inherit from parent rules
overrides: ["forbidden-0"]                # Optional - override specific inherited rules
---

# Rule definitions in markdown
FORBIDDEN: `TODO` - Convert TODOs to proper issue tracking
REQUIRED: `#[test]` - All functions must have tests
STANDARD: `unwrap()` - Prefer proper error handling over panics
CONVENTION: `snake_case` - Use snake_case for variable names
```

### Rule Types
- **FORBIDDEN**: Patterns that must not appear in code (blocks commits if found)
- **REQUIRED**: Patterns that must be present (blocks commits if missing)
- **STANDARD**: Recommended patterns (warnings only)
- **CONVENTION**: Style guidelines (informational)

### Rule Inheritance
Rules inherit from parent directories automatically. A `.synapse/` directory in `src/` will apply rules from both `src/.synapse/` and `./.synapse/`, with child rules taking precedence.

Directory structure example:
```
project/
├── .synapse/security.md          # Global security rules
├── src/
│   ├── .synapse/rust-patterns.md # Rust-specific rules (inherits from global)
│   └── main.rs                   # Validated against both security + rust-patterns
└── tests/
    ├── .synapse/test-standards.md # Test-specific rules
    └── integration.rs             # Validated against security + test-standards
```

### Document Indexing Format
For knowledge graph indexing, markdown files use this frontmatter:

```yaml
---
mcp: synapse          # Required - marks document for indexing
type: decision        # Optional - node type (rule, decision, architecture, component, function)
title: "Title"        # Optional - display name
tags: ["tag1"]        # Optional - categorization tags
---
```

**Supported node types:**
- `rule` - Development rules and guidelines
- `decision` - Architecture decisions and rationale  
- `architecture` - System architecture documentation
- `component` - Component specifications
- `function` - Function/method documentation
- Default: `file` (if no type specified)

## Automation & Hooks

Synapse MCP provides full automation through a **dual-hook system**:

**Proactive (Pre-Write)**: Validates AI-generated content BEFORE writing to files
**Reactive (Pre-Commit)**: Final safety net that catches manual edits and edge cases

This ensures 100% rule compliance while maintaining development velocity.

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

### Pre-Write Validation Hook
Proactive enforcement that validates content BEFORE writing to files:

```bash
# Test the pre-write hook manually
./hooks/claude-pre-write.sh src/main.rs "// TODO: implement this"

# Example with violations - will show auto-fixes
./hooks/claude-pre-write.sh src/api.js 'console.log("debug"); // TODO: remove'

# Configure Claude Code to use pre-write hook
export CLAUDE_PRE_WRITE_HOOK="./hooks/claude-pre-write.sh"
```

**Pre-Write Hook Flow:**
1. Claude Code attempts to write content to a file
2. Hook intercepts and sends content to `/enforce/pre-write` endpoint  
3. Synapse validates against applicable rules for that file path
4. If violations found:
   - High-confidence auto-fixes are applied automatically
   - Manual fixes required for complex violations
5. Only compliant content gets written to the file

**Auto-Fix Examples:**
- `TODO` → `// Issue #XXX:`
- `console.log` → `log::info!`
- `unwrap()` → `?` (when context allows)

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