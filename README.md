# Synapse AI Workspace Framework

**Transform your codebase into an AI-readable knowledge base that actively guides development**

Synapse is a comprehensive AI workspace framework that automatically builds intelligent project context from your documentation, enabling AI coding assistants like Claude Code to provide highly relevant, project-specific guidance.

## 🚀 Quick Start for Claude Code

### Option 1: One-Command Setup (Recommended)
```bash
# Build and initialize the workspace
cargo build --release
./target/release/synapse_mcp init --template=rust --hooks

# Start coding with AI context!
./target/release/synapse_mcp context --scope=all
```

### Option 2: Step-by-Step Setup
```bash
# 1. Build the Synapse CLI
cargo build --release

# 2. Initialize your project workspace
./target/release/synapse_mcp init MyProject --template=rust

# 3. Install automation hooks (optional but recommended)
./target/release/synapse_mcp init --hooks

# 4. Generate AI context for Claude
./target/release/synapse_mcp context --scope=all
```

## 📋 Claude Code Integration Guide

### Step 1: Install & Build Synapse
```bash
git clone <your-synapse-repo>
cd synapse_mcp
cargo build --release

# Add to your PATH (optional)
export PATH="$PWD/target/release:$PATH"
```

### Step 2: Initialize Your Project
```bash
# In your project directory
synapse init --template=rust  # or python, typescript, generic

# This creates:
# .synapse/rules/           - Coding standards & guidelines  
# .synapse/architecture/    - System design documentation
# .synapse/decisions/       - Architecture decision records
# .synapse/components/      - Component specifications
```

### Step 3: Fill in Your Project Documentation
Edit the generated templates in `.synapse/` with your project-specific information:

```bash
# Edit the key files:
$EDITOR .synapse/rules/coding_standards.md
$EDITOR .synapse/architecture/overview.md  
$EDITOR .synapse/rules/testing_strategy.md
```

**Example coding standards template:**
```yaml
---
mcp: synapse
type: rule
title: "MyProject Coding Standards"
tags: ["rust", "standards", "performance"]
---

# MyProject Coding Standards

## Performance Requirements
- All API responses must complete within 100ms
- Database queries must use connection pooling
- Memory usage should not exceed 512MB under normal load

## Error Handling
- Use `anyhow::Result` for application errors
- Never use `unwrap()` in production code
- Log all errors with structured logging
```

### Step 4: Generate Context for Claude Code
```bash
# Generate comprehensive context
synapse context --scope=all -o .synapse_context

# Or generate focused context for specific tasks:
synapse context --scope=test -o .synapse_test_context     # Testing context
synapse context --scope=api -o .synapse_api_context       # API development  
synapse context --scope=rules -o .synapse_rules_context   # Coding standards
```

### Step 5: Use in Claude Code Sessions

When starting a Claude Code session, the generated context file (`.synapse_context`) will be automatically loaded, providing Claude with:

- ✅ **Project-specific coding standards**
- ✅ **Architecture decisions and constraints** 
- ✅ **Testing requirements and patterns**
- ✅ **Performance guidelines and benchmarks**
- ✅ **Security practices and requirements**

## 🎯 Advanced Usage

### Automatic Context Updates
Enable git hooks for automatic context updates:
```bash
synapse init --hooks

# Now context auto-updates on every commit!
git add .synapse/rules/new-rule.md
git commit -m "Add new performance rule"
# Context automatically regenerated
```

### Task-Specific Contexts
Generate focused context for specific development tasks:

```bash
# API development context
synapse context --scope=api --format=markdown

# Testing and quality context  
synapse context --scope=test --format=json

# Architecture and design context
synapse context --scope=architecture --format=plain
```

### Project Health Monitoring
```bash
# Check system status
synapse status --verbose

# Query your knowledge base
synapse query "What are our performance requirements?"
synapse query "How should I handle errors in this project?"
```

## 🏗️ Architecture

### Core Components
- **Unified CLI**: Single `synapse` command for all operations
- **Project Templates**: Language-specific documentation scaffolding
- **Smart Context Generation**: Scope-based filtering for relevant context
- **Parallel Processing**: High-performance document indexing
- **Knowledge Graph**: Optional Neo4j integration for advanced querying

### Data Flow
```
Documentation → Synapse Templates → AI Context → Claude Code
     ↓              ↓                  ↓           ↓
  .synapse/     Validation       .synapse_     Enhanced AI
  templates      & Parsing        context      Guidance
```

## 🎨 Available Templates

### Rust Projects (`--template=rust`)
- Rust-specific coding standards
- Performance optimization guidelines  
- Error handling best practices
- Security and memory safety rules

### Python Projects (`--template=python`)
- PEP 8 compliance guidelines
- Type hint requirements
- Virtual environment management
- Testing with pytest patterns

### TypeScript Projects (`--template=typescript`)  
- ESLint and Prettier configurations
- Type definitions and interfaces
- Bundle optimization guidelines
- Modern testing patterns

### Generic Projects (`--template=generic`)
- Universal coding standards
- Architecture documentation templates
- Testing strategy frameworks
- Decision record (ADR) templates

## 📊 Context Scopes

| Scope | Description | Best For |
|-------|-------------|----------|
| `all` | Complete project context | General development, onboarding |
| `rules` | Coding standards & guidelines | Code review, standards enforcement |
| `architecture` | System design & decisions | Architecture planning, refactoring |
| `test` | Testing strategies & patterns | Writing tests, QA work |
| `api` | API design & documentation | Backend development, API design |
| `security` | Security practices & requirements | Security review, compliance |

## ⚡ Performance

- **Indexing**: <500ms for typical documentation sets
- **Context Generation**: <200ms for focused scopes
- **Parallel Processing**: Automatic optimization for large codebases
- **Smart Caching**: Context regenerated only when documents change

## 🔧 Configuration

### Environment Variables
```bash
# Optional Neo4j integration
NEO4J_URI=bolt://localhost:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=password

# Synapse configuration
SYNAPSE_VERBOSE=true          # Enable detailed logging
SYNAPSE_CONTEXT_FILE=.synapse_context  # Default context file name
```

### Document Format
All Synapse documents use YAML frontmatter for metadata:

```yaml
---
mcp: synapse              # Required: marks document for Synapse
type: rule                # Document type (rule, architecture, decision, component)
title: "Document Title"   # Human-readable title
tags: ["tag1", "tag2"]   # Categorization tags
---

# Your documentation content here
```

## 🤖 Claude Code Tips

### Best Practices
1. **Start each session** with `synapse context --scope=all` for comprehensive guidance
2. **Use focused scopes** when working on specific features (e.g., `--scope=test` for testing)
3. **Update documentation templates** as your project evolves
4. **Enable git hooks** for automatic context updates

### Common Workflows
```bash
# Starting API development
synapse context --scope=api
# Claude now knows your API standards, error handling, and performance requirements

# Code review preparation  
synapse context --scope=rules
# Claude can check code against your specific standards

# Architecture planning
synapse context --scope=architecture  
# Claude understands your system design and constraints
```

## 🚨 Troubleshooting

### Context Not Loading
```bash
# Check system status
synapse status --verbose

# Regenerate context
synapse context --scope=all --format=markdown

# Verify file exists
ls -la .synapse_context
```

### Performance Issues
```bash
# Use parallel processing for large doc sets
synapse index docs/*.md --parallel 8

# Check indexing performance
SYNAPSE_VERBOSE=1 synapse context --scope=all
```

### Template Issues
```bash
# Re-initialize templates
synapse init --template=rust  # Regenerates templates

# Check template structure
find .synapse -name "*.md" -exec head -10 {} \;
```

## 🎉 Success Indicators

You'll know Synapse is working when Claude Code:
- ✅ **Follows your coding standards** automatically
- ✅ **Suggests project-appropriate patterns** and solutions
- ✅ **Remembers architectural decisions** and constraints
- ✅ **Applies consistent error handling** and testing approaches
- ✅ **Respects performance requirements** and security practices

## 💡 Pro Tips

1. **Start simple**: Use generic templates first, then customize
2. **Be specific**: Include exact performance numbers, not vague requirements
3. **Update regularly**: Keep documentation current with code changes
4. **Use scopes**: Generate focused context for specific development tasks
5. **Enable automation**: Git hooks ensure context stays synchronized

---

**Ready to transform your AI coding experience?** Run `synapse init` in your project directory and watch Claude Code become your perfect coding partner! 🚀