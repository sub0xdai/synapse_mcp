
````markdown
# Implementation Plan: Dual-Hook GraphRAG Enforcement System

## Core Architecture

Build a system with two complementary hooks:
1.  **Write Hook:** Enforces rules when code is created/modified.
2.  **Read Hook:** Provides context and examples when Claude requests information.

---

## System Components

### 1. Distributed Rule Files
- `.synapse.md` files in each directory (like your GEMINI.md).
- **Inheritance hierarchy:** Child directories inherit parent rules.
- **Override capability:** Specific directories can override parent rules with `@overrides`.
- **Project-specific configuration:** Each project defines its own invariants.

### 2. GraphRAG Knowledge Engine
A dynamic graph that understands relationships between rules.

```rust
pub struct RuleGraph {
    nodes: HashMap<PathBuf, RuleNode>,
    edges: Vec<RuleRelationship>,
    index: PatternIndex,
}

impl RuleGraph {
    // Build graph from distributed .synapse.md files
    pub fn from_project(root: &Path) -> Self {
        // Recursively discover all rule files
        // Parse relationships (@inherits, @overrides, @enforces)
        // Build pattern index for fast matching
    }

    // Get applicable rules for a given file path
    pub fn rules_for(&self, path: &Path) -> CompositeRules {
        // Walk up directory tree collecting rules
        // Apply inheritance and overrides
        // Return merged ruleset
    }
}
````

### 3\. Pattern Enforcer MCP Server

```rust
pub struct PatternEnforcer {
    graph: RuleGraph,
    cache: ContextCache,
}

impl MCPServer for PatternEnforcer {
    // Pre-action enforcement
    async fn before_action(&self, action: Action) -> EnforcementResult {
        match action {
            Action::Create(path) => {
                // Check TDD requirements
                // Verify architecture constraints
                // Return required patterns
            }
            Action::Edit(path, content) => {
                // Validate against forbidden patterns
                // Check style requirements
                // Suggest alternatives
            }
        }
    }

    // Post-action validation
    async fn validate(&self, changes: &Changes) -> ValidationResult {
        // Check for violations
        // Verify required patterns are present
        // Return actionable feedback
    }
}
```

-----

## 4\. Example Rule File Structure

Using `GEMINI.md` as a template:

### `PROJECT_ROOT/.synapse.md`

```markdown
@project: minizinc-introspector
@constant: Feinburhm_Constant

## Core Philosophy
- **Monotonic Development**: Add-only, never edit
- **Immutable History**: All evolution via new modules
- **Direct Edits**: FORBIDDEN - violation of Feinburhm Constant

## Coding Standards
- **Logging**: MANDATORY use `gemini_utils::gemini_eprintln!`
- **Standard eprintln**: FORBIDDEN
- **Kantspel System**:
  - NO literal `\n`, `{}`, `{{}}`
  - USE ✨ for newline, 🧱 for braces

## Git Workflow
- **Commits**: MUST use `git commit -F temp_commit_message.txt`
- **Never**: `cargo clean` or `cargo update`

## AI Directives
- **Edit Tool**: FORBIDDEN - rewrite instead
- **Replace Tool**: Last resort only
- **Role**: Human augmentation, not automation
```

### `src/gemini_utils/.synapse.md`

```markdown
@inherits: ../../.synapse.md
@module: gemini_utils

## Module-Specific Rules
- **Primary Function**: `gemini_eprintln!` macro
- **Character Translation**:
  - ✨ → \n
  - 🧱 → {}
  - 🎯 → {{}}

## Required Patterns
- All logging MUST go through this module
- Direct println! calls will be rejected
```

-----

## 5\. Hook Integration

**Pre-commit Hook (Write Path):**

```bash
#!/bin/bash
# Check all changed files against rules
synapse-enforce check --files $(git diff --cached --name-only)
```

**Claude Hook (Read Path):**

```yaml
# .claude-hooks.yaml
on_file_open:
  action: load_context
  command: synapse-enforce context --path $FILE

before_edit:
  action: check_rules
  command: synapse-enforce validate --action edit --path $FILE

after_edit:
  action: verify_compliance
  command: synapse-enforce verify --diff $CHANGES
```

-----

## 6\. Progressive Enforcement

```yaml
enforcement_modes:
  strict: # Your GEMINI project
    - monotonic: block_any_edit
    - kantspel: auto_translate_chars
    - logging: reject_standard_eprintln

  standard: # Normal projects
    - tdd: warn_without_test
    - patterns: suggest_improvements

  learning: # New developers
    - show_examples: true
    - explain_violations: true
```

-----

## 7\. Implementation Steps

1.  **Create CLI tool (`synapse-enforce`) that:**
      - Discovers and parses `.synapse.md` files.
      - Builds the rule graph.
      - Provides an enforcement API.
2.  **Build MCP server that:**
      - Integrates with Claude Code.
      - Intercepts actions.
      - Provides real-time feedback.
3.  **Deploy rule templates for common patterns:**
      - TDD enforcement.
      - Architecture layering.
      - Project-specific (like your GEMINI rules).
4.  **Create example `.synapse.md` files for:**
      - Root project rules.
      - Source code standards.
      - Test requirements.
      - Module-specific rules.

-----

## Key Benefits

1.  **Project-Specific Enforcement:** Each project defines its own invariants.
2.  **Directory-Scoped Rules:** Different rules for different parts of the codebase.
3.  **GraphRAG Intelligence:** Understands relationships between rules.
4.  **Progressive Adoption:** Teams can start gentle and increase strictness.
5.  **Living Documentation:** Rules evolve with the project.

This system ensures that as codebases expand, they maintain their philosophical and technical integrity through active, intelligent enforcement.

```
```
