
# Synapse MCP

A dynamic memory system for AI coding assistants that provides persistent project context through automated documentation indexing and rule enforcement.

## Core Features

* **Rule Enforcement**: Validates code against project-specific standards using `.synapse.md` files
* **AI Context Generation**: Provides structured project context to AI assistants like Claude
* **Inheritance System**: Directory-based rule inheritance with override capabilities  
* **MCP Server**: High-performance API server for real-time AI integration
* **Git Integration**: Pre-commit hooks for automatic rule checking

-----

## Quick Start

### 1. Deploy with Docker

The fastest way to get started is using Docker Compose:

```bash
git clone <your-synapse-repo>
cd synapse_mcp

# Start Neo4j database and Synapse MCP server
docker-compose up -d

# Verify services are running
docker-compose ps
```

This starts:
- **Neo4j** on `localhost:7474` (browser) and `localhost:7687` (bolt)
- **Synapse MCP Server** on `localhost:8080`

### 2. Create Your First Rules

Copy the example rules to your project and customize them:

```bash
# Copy example rules to your project root
cp rules_examples/.synapse.md .
cp -r rules_examples/src .

# Or start with specific examples
cp rules_examples/.synapse.md my_project/.synapse.md
```

### 3. Test Rule Enforcement

Check files against your rules:

```bash
# Check specific files
./target/release/synapse_mcp check src/main.rs lib.rs

# Check with verbose output
./target/release/synapse_mcp check src/* --verbose

# Dry run to see what rules apply
./target/release/synapse_mcp check src/* --dry-run
```

### 4. Generate AI Context

Get structured context for AI assistants:

```bash
# Generate context for specific file
./target/release/synapse_mcp enforce-context src/main.rs

# Generate in different formats
./target/release/synapse_mcp enforce-context src/main.rs --format json
./target/release/synapse_mcp enforce-context src/main.rs --output .context.md
```

### 5. Setup Git Hooks (Optional)

Automate rule checking on commits:

```bash
# One-time setup
./scripts/setup-hooks.sh

# Test the hook
git add src/main.rs
git commit -m "Test commit"  # Rules will be automatically checked
```

-----

## Writing Rules

Rules are defined in `.synapse.md` files placed throughout your project directory structure. See **[DOCS_RULES.md](DOCS_RULES.md)** for complete documentation on writing rules.

### Quick Reference

```yaml
---
mcp: synapse          # Required - marks file for Synapse MCP
type: rule            # Optional - node type
inherits: ["../.synapse.md"]  # Optional - inherit from parent
overrides: ["forbidden-0"]    # Optional - override specific rules
---

# Rule Examples

FORBIDDEN: `TODO` - Convert TODOs to proper issue tracking
REQUIRED: `#[test]` - All functions must have tests  
STANDARD: `unwrap()` - Prefer proper error handling
CONVENTION: `snake_case` - Use snake_case for variables
```

### Rule Inheritance

```
project/
├── .synapse.md           # Root rules (global)
├── src/
│   ├── .synapse.md       # Inherits root + src-specific rules
│   └── api/
│       └── .synapse.md   # Inherits root + src + api-specific rules
```

-----

## Deployment

### Production Deployment

For production use, deploy with Docker Compose:

```bash
# Clone and build
git clone <repository>
cd synapse_mcp

# Start services in background
docker-compose up -d

# View logs
docker-compose logs -f synapse-server
docker-compose logs -f neo4j

# Scale or restart services
docker-compose restart synapse-server
docker-compose down && docker-compose up -d
```

### Service Endpoints

Once deployed:
- **Neo4j Browser**: http://localhost:7474 (username: `neo4j`, password: `password`)
- **MCP Server API**: http://localhost:8080
- **Health Check**: `curl http://localhost:8080/health`

### Environment Variables

Configure via `.env` file or environment:

```bash
# Neo4j Configuration
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

# Synapse Configuration
SYNAPSE_VERBOSE=true              # Enable verbose logging
SYNAPSE_CONTEXT_FILE=.context.md  # Default context file name
RUST_LOG=info                     # Rust logging level
```

### Development Setup

For development without Docker:

```bash
# Install dependencies
cargo build --release

# Start Neo4j manually (or use Docker)
docker run -p 7474:7474 -p 7687:7687 -e NEO4J_AUTH=neo4j/password neo4j:5-community

# Run MCP server locally
./target/release/synapse_mcp serve --enable-enforcer --port 8080
```

-----

## CLI Reference

| Command | Description | Example |
| :--- | :--- | :--- |
| `check` | Validate files against rules | `synapse check src/*.rs --verbose` |
| `enforce-context` | Generate AI context for path | `synapse enforce-context src/main.rs` |
| `serve` | Start MCP server | `synapse serve --enable-enforcer` |
| `status` | Show system status | `synapse status` |
| `init` | Initialize project templates | `synapse init --template rust` |

### Rule Enforcement Commands

```bash
# Check files (Write Hook)
synapse check src/main.rs src/lib.rs        # Check specific files
synapse check src/* --verbose               # Check with details
synapse check . --dry-run                   # Preview without failing

# Generate context (Read Hook)  
synapse enforce-context src/api.rs          # Context for specific file
synapse enforce-context . --format json     # JSON format context
synapse enforce-context . --output ctx.md   # Save to file
```

### Server Commands

```bash
# Start MCP server
synapse serve                                # Basic server
synapse serve --enable-enforcer             # With rule enforcement
synapse serve --port 3000 --host 0.0.0.0   # Custom host/port

# Check status
synapse status                               # System status
curl http://localhost:8080/health           # API health check
```

-----

## Architecture

Synapse MCP uses a dual-hook architecture:

* **Write Hook**: Pre-commit validation using `PatternEnforcer` and `RuleGraph`
* **Read Hook**: Real-time AI context generation via MCP server
* **Knowledge Graph**: Neo4j stores project documentation and relationships

### Core Components

- **RuleGraph**: In-memory rule inheritance and relationship tracking
- **PatternEnforcer**: Rule enforcement and context generation engine  
- **MCP Server**: High-performance API with Axum framework
- **Rule Discovery**: Recursive scanning and parsing of `.synapse.md` files

For detailed architecture documentation, see **[architecture.md](architecture.md)**.
