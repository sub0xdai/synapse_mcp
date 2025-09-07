
````markdown
# Implementation Plan: Dual-Hook GraphRAG Enforcement System

## Core Architecture

Build a system with two complementary hooks:
1.  **Write Hook:** Enforces rules when code is created/modified.
2.  **Read Hook:** Provides context and examples when Claude requests information.

---

## System Components

### 1. ✅ Distributed Rule Files (IMPLEMENTED)
- `.synapse/` directories containing any `.md` files (security.md, performance.md, etc.)
- **Flexible Organization:** Multiple rule files per directory for domain-specific rules
- **Inheritance hierarchy:** Child directories inherit parent rules via directory traversal
- **Override capability:** Specific directories can override parent rules with `@overrides`
- **Project-specific configuration:** Each project defines its own invariants

### 2. ✅ RuleSystem Engine (IMPLEMENTED)
A robust system that efficiently manages rule relationships and inheritance.

```rust
pub struct RuleSystem {
    discovery: RuleDiscovery,
    parser: RuleParser,
}

impl RuleSystem {
    // ✅ IMPLEMENTED: Load rules from .synapse/*.md files
    pub fn load_rules(root: &Path) -> Vec<RuleSet> {
        // Recursively discover all .md files in .synapse/ directories
        // Parse YAML frontmatter and rule definitions
        // Support multiple files per directory
    }

    // ✅ IMPLEMENTED: Get applicable rules with directory mapping
    pub fn rules_for_path(&self, path: &Path, rule_sets: &[RuleSet]) -> CompositeRules {
        // Create HashMap<PathBuf, Vec<&RuleSet>> for fast lookup
        // Walk up directory tree with canonicalized paths
        // Apply inheritance and overrides
        // Return merged ruleset with proper precedence
    }
}
```

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

## 7. Implementation Status

### Phase 1: Core Engine (COMPLETED)

1.  **✅ CLI tool (`synapse_mcp`) implemented:**
      - Discovers and parses all `.md` files in `.synapse/` directories.
      - Unified parser supporting all rule types.
      - Robust `rules_for_path` logic with directory mapping and inheritance.
      - Core commands: `check`, `enforce-context`, `server`, `init`.
      
2.  **✅ Rule enforcement system working:**
      - Real-time violation detection.
      - Proper error reporting with line numbers and context.

### Phase 2: Production Hardening (COMPLETED)

1.  **✅ Pre-Write Hook & Security:**
    - Implemented a secure `claude-pre-write.sh` hook to prevent shell injection and resource leaks.
    - Added the `/enforce/pre-write` endpoint for real-time validation.

2.  **✅ Server Authentication:**
    - Implemented optional, constant-time secure bearer token authentication for all sensitive endpoints.

3.  **✅ Comprehensive Error Handling:**
    - Replaced all `unwrap()` calls in production code with a centralized `SynapseError` type.
    - Implemented `IntoResponse` for `SynapseError` to provide structured JSON error responses with correct HTTP status codes.

4.  **✅ Safe AST-Based Auto-Fix:**
    - Replaced naive regex auto-fixes with a safe, AST-based system using the `syn` crate.
    - Auto-fixes for `unwrap()` are now context-aware and opt-in via a feature flag.

5.  **✅ Performance Enhancements:**
    - **Rule Caching:** Implemented a high-performance, in-memory rule cache using `moka` to drastically reduce filesystem I/O.
    - **DB Connection Pooling:** Integrated `bb8` to manage a pool of Neo4j connections, improving performance and reliability under load.

6.  **✅ Test Infrastructure:**
    - Created a hermetic `TestProject` helper for isolated, reliable filesystem tests.
    - Refactored key test modules to use the new infrastructure, fixing underlying bugs in the process.

### Phase 3: Next Steps (IN PROGRESS)

1.  **Complete Test Refactoring:** Finish migrating all remaining test files to use the new `TestProject` helper.
2.  **Deployment Automation:** Build the `curl | sh` installer script with cross-platform binary support.
3.  **Developer Experience:** Implement rule suppression, severity levels, and IDE integration.

-----

## Key Benefits ACHIEVED

1.  **✅ Project-Specific Enforcement:** Each project defines its own invariants via `.synapse/` directories.
2.  **✅ Directory-Scoped Rules:** Different rules for different parts of the codebase with inheritance.
3.  **✅ Flexible Organization:** Multiple rule files per domain (security.md, performance.md, etc.).
4.  **✅ Robust & Reliable:** The server is now hardened with comprehensive error handling, connection pooling, and a secure authentication system.
5.  **✅ High Performance:** Rule caching and connection pooling provide sub-millisecond response times for repeated operations.
6.  **✅ Intelligent & Safe:** The system can now perform complex, context-aware analysis and auto-fixes using ASTs.
7.  **✅ Production Ready:** The core server is now secure, observable, reliable, and performant, making it suitable for production deployments.

```
```
