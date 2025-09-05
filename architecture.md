# Architecture: Synapse MCP with PatternEnforcer

## 1. Core Concept

Synapse MCP operates with a dual-hook architecture for intelligent rule enforcement:
- **Write Hook (PatternEnforcer):** Pre-commit validation against project rules
- **Read Hook (Context Generation):** Real-time AI context with applicable rules
- **Knowledge Graph:** Neo4j integration for advanced querying and relationships

## 2. Technology Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| PatternEnforcer | Rust | Real-time rule validation and context generation |
| RuleGraph | Rust | In-memory rule inheritance and relationship tracking |
| MCP Server | Rust (Axum Framework) | High-performance API with enforcement endpoints |
| Core Logic/Indexer | Rust | Performance-critical parsing and indexing |
| Knowledge Graph | Neo4j | Storing and querying connected data |
| Hook Management | pre-commit Framework | User-friendly git hook installation |

## 3. PatternEnforcer Architecture

### RuleGraph Engine
```rust
.synapse.md files → RuleDiscovery → RuleParser → RuleGraph
                                                      ↓
File Path Query → Inheritance Resolution → CompositeRules → PatternEnforcer
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
  - File validation against FORBIDDEN/REQUIRED patterns
  - Multi-format context generation (Markdown/JSON/Plain)
  - Integration with MCP server endpoints

#### Rule Discovery (`src/rules/discovery.rs`)
- **Purpose:** Recursive discovery of `.synapse.md` files
- **Features:**
  - Efficient directory traversal
  - Markdown file filtering
  - YAML frontmatter validation

## 4. Data Flow Architecture

### Write Path (Rule Enforcement)
```
Developer → git commit → pre-commit hook → PatternEnforcer.check_files()
                                              ↓
RuleGraph.rules_for(file_path) → CompositeRules → Pattern Matching → Violations/Success
```

### Read Path (AI Context Generation)
```
Claude Code → context hook → MCP Server → PatternEnforcer.generate_context()
                                              ↓
RuleGraph.rules_for(file_path) → Rule Formatting → AI-Ready Context
```

### Neo4j Integration Path
```
Markdown files → Indexer → Neo4j DB → MCP Server → Natural Language Queries
```

## 5. MCP Server Endpoints

### Standard Endpoints
- `GET /health` - Server health check
- `POST /query` - Natural language queries to knowledge graph
- `GET /nodes/:type` - Query nodes by type
- `GET /node/:id/related` - Find related nodes

### PatternEnforcer Endpoints (when enabled)
- `POST /enforce/check` - Validate files against rules (Write Hook)
- `POST /enforce/context` - Generate AI context for file path (Read Hook)
- `POST /rules/for-path` - Get applicable rules for a specific path

## 6. Rule File Format

### .synapse.md Structure
```yaml
---
mcp: synapse                    # Required marker
type: rule                      # Optional node type
inherits: ["../parent/.synapse.md"]  # Optional inheritance
overrides: ["old-rule-id"]      # Optional rule overrides
tags: ["performance", "api"]    # Optional categorization
---

# Rule Content

FORBIDDEN: `println!` - Use logging framework instead.
REQUIRED: `#[test]` - All functions must have tests.
STANDARD: `unwrap` - Consider proper error handling.
CONVENTION: `snake_case` - Use snake_case for variables.
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
