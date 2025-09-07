# Architecture: Synapse MCP with PatternEnforcer

## 1. Core Concept

Synapse MCP operates with a dual-hook architecture for intelligent rule enforcement:
- **Write Hook (PatternEnforcer):** Pre-commit validation against project rules.
- **Pre-Write Hook (Real-time Validation):** Intercepts code generation to validate and auto-fix content before it is written to disk.
- **Knowledge Graph:** Neo4j integration for advanced querying and relationships.

## 2. Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| PatternEnforcer | Rust | Real-time rule validation and context generation |
| RuleGraph | Rust | In-memory rule inheritance and relationship tracking |
| MCP Server | Rust (Axum Framework) | High-performance API with enforcement endpoints |
| Caching Layer | Rust (Moka) | High-performance, thread-safe rule caching |
| DB Connection Pool | Rust (bb8) | Asynchronous database connection pooling |
| Core Logic/Indexer | Rust | Performance-critical parsing and indexing |
| Knowledge Graph | Neo4j | Storing and querying connected data |
| Hook Management | pre-commit Framework | User-friendly git hook installation |

## 3. PatternEnforcer Architecture

### RuleGraph Engine
```rust
.synapse/*.md files → RuleDiscovery → RuleParser → RuleSystem
                                                      ↓
File Path Query → Directory Mapping → rules_for_path → CompositeRules → PatternEnforcer
```

### Key Components

#### RuleGraph (`src/rule_graph.rs`)
- **Purpose:** In-memory graph of rule relationships
- **Features:** 
  - Directory-based inheritance
  - Explicit override support
  - Fast path-to-rules resolution
  - Cycle detection

#### PatternEnforcer (`src/mcp_server/pattern_enforcer.rs`)
- **Purpose:** Rule enforcement and context generation
- **Capabilities:**
  - Dual-mode analysis: Fast regex matching for simple patterns and AST-based analysis for complex, context-aware rules (e.g., safe auto-fixes).
  - File validation against FORBIDDEN/REQUIRED patterns.
  - Multi-format context generation (Markdown/JSON/Plain).

#### RuleCache (`src/cache.rs`)
- **Purpose:** High-performance, in-memory caching for resolved rule sets.
- **Features:**
  - Drastically reduces filesystem I/O for repeated requests.
  - Uses Moka for a thread-safe, async-compatible cache.
  - Configurable TTL and max entry limits.

#### ConnectionPool (`src/db.rs`)
- **Purpose:** Manages a pool of database connections.
- **Features:**
  - Prevents resource exhaustion by limiting and reusing connections.
  - Uses bb8 for asynchronous connection pooling.
  - Includes health monitoring for database connections.

#### Rule Discovery (`src/rules/discovery.rs`)
- **Purpose:** Recursive discovery of `.md` files in `.synapse/` directories
- **Features:**
  - Efficient directory traversal scanning for `.synapse/` directories
  - Discovers ALL `.md` files within `.synapse/` directories  
  - Supports multiple rule files per directory (security.md, performance.md, etc.)
  - YAML frontmatter validation with `mcp: synapse` marker

## 4. Data Flow Architecture

### Write Path (Rule Enforcement)
```
Developer → git commit → pre-commit hook → PatternEnforcer.check_files()
                                              ↓
RuleGraph.rules_for(file_path) → CompositeRules → Pattern Matching → Violations/Success
```

### Pre-Write Path (Real-time Validation)
```
AI Assistant → Pre-Write Hook → MCP Server (/enforce/pre-write) → PatternEnforcer.validate_pre_write()
                                                                        ↓
                                    (RuleGraph + Cache) → CompositeRules → AST/Regex Matching → {valid: bool, fixed_content?: string}
```

### Neo4j Integration Path
```
Markdown files → Indexer → Neo4j DB → MCP Server → Natural Language Queries
```

## 5. MCP Server Endpoints

### Security

When configured with a `SYNAPSE_AUTH_TOKEN`, all `/enforce/*` and `/query` endpoints are protected. Clients must provide a valid `Authorization: Bearer <token>` header with all requests.

### Standard Endpoints
- `GET /health` - Server health check (public).
- `POST /query` - Natural language queries to knowledge graph.
- `GET /nodes/:type` - Query nodes by type.
- `GET /node/:id/related` - Find related nodes.

### PatternEnforcer Endpoints (when enabled)
- `POST /enforce/pre-write` - Real-time validation of in-memory content with auto-fix capabilities.
- `POST /enforce/check` - Validate saved files against rules.
- `POST /enforce/context` - Generate AI context for file path.
- `POST /rules/for-path` - Get applicable rules for a specific path.

## 6. Rule File Format

### .synapse/*.md Structure
Rules are now organized in `.synapse/` directories, with any `.md` filename:

```yaml
---
mcp: synapse                    # Required marker
type: rule                      # Optional node type
title: "Security Rules"         # Optional display name
inherits: ["../parent/.synapse/security.md"]  # Optional inheritance
overrides: ["old-rule-id"]      # Optional rule overrides
tags: ["security", "compliance"]    # Optional categorization
---

# Rule Content (Examples by Domain)

FORBIDDEN: `password` - Never hardcode passwords in source code.
REQUIRED: `validate_input` - All user inputs must be validated.
STANDARD: `https://` - Use HTTPS for external communications.
CONVENTION: `auth_` - Authentication functions should be prefixed with auth_.
```

### Flexible Organization
```
project/
├── .synapse/
│   ├── security.md      # Security rules
│   ├── performance.md   # Performance rules  
│   └── coding.md        # General coding standards
├── src/
│   └── .synapse/
│       └── rust.md      # Rust-specific patterns
└── tests/
    └── .synapse/
        └── testing.md   # Test-specific rules
```

### Rule Types & Enforcement
- **FORBIDDEN**: Patterns that block commits (exit 1)
- **REQUIRED**: Patterns that must be present (exit 1)
- **STANDARD**: Suggestions shown to AI (non-blocking)
- **CONVENTION**: Style guidelines shown to AI (non-blocking)

## 7. Hook Integration

### Pre-commit Hook (`scripts/pre-commit-hook.sh`)
```bash
synapse check --files $(git diff --cached --name-only)
```

### Claude Context Hook (`scripts/claude-context-hook.sh`)
```bash
# Start MCP server with PatternEnforcer
synapse serve --enable-enforcer --port 8080

# Generate context via API or CLI
curl -X POST /enforce/context -d '{"path": "src/main.rs"}'
```

### Setup Script (`scripts/setup-hooks.sh`)
- Automated installation of both hooks
- Pre-commit framework integration  
- Shell environment configuration
- Testing and validation

## 8. Performance Characteristics

- **Rule Resolution:** <50ms for typical project hierarchies
- **File Checking:** <100ms per file for pattern matching
- **Context Generation:** <200ms for complete rule context
- **MCP Server:** <100ms average API response time
- **Pre-commit Hook:** <500ms total for typical change sets

## 9. Extensibility Points

### Custom Rule Types
Add new rule types in `src/models.rs`:
```rust
pub enum RuleType {
    Forbidden,
    Required,
    Standard,
    Convention,
    Custom(String),  // Extensible
}
```

### Custom Enforcement Logic
Extend `PatternEnforcer::check_file_against_rules()` for domain-specific validation.

### Custom Context Formats  
Add new formats in `PatternEnforcer::generate_context()` method.

This architecture provides a robust, performant, and extensible foundation for intelligent rule enforcement in software development workflows.
