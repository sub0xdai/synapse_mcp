# Synapse Rules Documentation

This document explains how to use Synapse MCP to define and enforce project-specific rules. Rules are defined in `.md` files located within `.synapse/` directories.

## What are Synapse Rule Files?

Synapse rule files are Markdown documents with YAML frontmatter that define development rules for your project. They are placed inside directories named `.synapse/`. These rules apply to all files within the directory containing `.synapse/` and all of its subdirectories, following inheritance patterns.

## File Structure

### YAML Frontmatter (Required)

Every `.synapse.md` file must begin with YAML frontmatter:

```yaml
---
mcp: synapse          # Required - identifies file as Synapse rule file
type: rule            # Optional - node type (rule, decision, architecture)
title: "Rule Title"   # Optional - human-readable title
tags: ["tag1", "tag2"] # Optional - categorization tags
inherits: ["../parent/.synapse.md"] # Optional - inherit from parent files
overrides: ["rule-id"] # Optional - override specific inherited rules
---
```

### Required Fields
- `mcp: synapse` - Marks the file for processing by Synapse MCP

### Optional Fields
- `type` - Node type for knowledge graph (rule, decision, architecture, component, function)
- `title` - Display name for the rule set
- `tags` - Array of tags for categorization and filtering
- `inherits` - Array of paths to parent rule files to inherit from
- `overrides` - Array of rule IDs to override from inherited rules

## Rule Types and Syntax

### FORBIDDEN Rules
Block commits and cause enforcement failures:
```
FORBIDDEN: `pattern` - Explanation message
```
Example:
```
FORBIDDEN: `TODO` - Convert TODO comments to proper issue tracking
FORBIDDEN: `console.log` - Use structured logging framework instead
```

### REQUIRED Rules
Must be present for enforcement to pass:
```
REQUIRED: `pattern` - Explanation message
```
Example:
```
REQUIRED: `SPDX-License-Identifier` - All files must have license header
REQUIRED: `#[test]` - Public functions must have unit tests
```

### STANDARD Rules
Suggestions shown to AI assistants (non-blocking):
```
STANDARD: `pattern` - Guidance message
```
Example:
```
STANDARD: `unwrap()` - Prefer proper error handling over unwrap
STANDARD: `async` functions should complete within 500ms
```

### CONVENTION Rules
Style guidelines shown to AI assistants (non-blocking):
```
CONVENTION: `pattern` - Style guidance
```
Example:
```
CONVENTION: `snake_case` - Use snake_case for Rust variables
CONVENTION: Add doc comments to public APIs
```

## Pattern Matching

Patterns can be:
- **Literal strings**: `TODO`, `console.log`
- **Regular expressions**: `println!\(.*\)`, `#\[derive\(.*\)\]`
- **Natural language descriptions**: "functions should complete within 500ms"

Note: Use proper escaping for regex special characters in patterns.

## Inheritance and Overrides

### Directory-Based Inheritance
Rules are automatically inherited from parent directories containing a `.synapse/` folder:
```
project/
├── .synapse/
│   └── global.md       # Root rules
├── src/
│   ├── .synapse/
│   │   └── rust.md     # Inherits root rules + adds src-specific rules
│   └── api/
│       └── main.rs     # This file inherits rules from both ../.synapse/ and ../../.synapse/
```

### Explicit Inheritance
Use the `inherits` field to explicitly specify parent rule files:
```yaml
---
mcp: synapse
inherits: ["../../global/.synapse.md", "../shared/.synapse.md"]
---
```

### Rule Overrides
Override inherited rules using the `overrides` field:
```yaml
---
mcp: synapse
inherits: ["../.synapse.md"]
overrides: ["todo-forbidden", "println-forbidden"]
---

# Override: Allow TODOs in development
STANDARD: `TODO` - TODOs allowed during development
```

## Complete Example

### Root Rules (`/.synapse.md`)
```yaml
---
mcp: synapse
type: rule
title: "Global Project Rules"
tags: ["global", "standards"]
---

# Global Development Standards

FORBIDDEN: `TODO` - Use issue tracking instead
FORBIDDEN: `console.log` - Use proper logging
REQUIRED: `SPDX-License-Identifier` - License headers required
STANDARD: `snake_case` - Follow naming conventions
```

### Source Rules (`/src/.synapse.md`)
```yaml
---
mcp: synapse
type: rule
title: "Source Code Rules"
tags: ["source", "development"]
inherits: ["../.synapse.md"]
overrides: ["todo-forbidden"]
---

# Development-Specific Rules

# Override: Allow TODOs in development
STANDARD: `TODO` - Track TODOs but don't block development

# Additional requirements for source code
REQUIRED: `#[test]` - All public functions need tests
FORBIDDEN: `panic!` - Use Result types for error handling
```

## Usage with Synapse MCP

### Rule Enforcement (Write Hook)
```bash
# Check files against rules before commit
synapse check src/main.rs src/lib.rs

# Dry run to see what rules would be applied
synapse check --dry-run --verbose src/
```

### AI Context Generation (Read Hook)
```bash
# Generate context for a specific file
synapse enforce-context src/api/users.rs

# Generate context in different formats
synapse enforce-context src/main.rs --format json
synapse enforce-context src/main.rs --output .context.md
```

### MCP Server Integration
When running the MCP server with `--enable-enforcer`, it provides these endpoints:
- `POST /enforce/pre-write` - Validates content in real-time before it is written to a file, providing instant feedback and auto-fixes.
- `POST /enforce/check` - Validate saved files against rules.
- `POST /enforce/context` - Generate AI context for file paths.
- `POST /rules/for-path` - Get applicable rules for specific paths.

**Note:** When authentication is enabled via `SYNAPSE_AUTH_TOKEN`, all `/enforce/*` endpoints require a valid Bearer Token.

## Best Practices

1. **Start with root rules** - Define global standards in your project root
2. **Use inheritance** - Let subdirectories inherit and override as needed
3. **Be specific with patterns** - Use precise regex patterns for reliable matching
4. **Document your rules** - Include clear explanations for why rules exist
5. **Tag your rules** - Use tags for easy filtering and organization
6. **Test your rules** - Use `synapse check --dry-run` to verify rule behavior
7. **Keep rules focused** - Separate different concerns into different rule files

## Troubleshooting

### Common Issues
- **Rules not applying**: Check that `mcp: synapse` is in the frontmatter
- **Inheritance not working**: Verify file paths in `inherits` array
- **Regex not matching**: Test patterns with `synapse check --verbose`
- **Performance issues**: Avoid overly complex regex patterns

### Debug Mode
Use verbose flags to see rule processing:
```bash
synapse check --verbose src/
SYNAPSE_VERBOSE=1 synapse check src/
```

This will show which rules are loaded, inheritance chains, and pattern matching details.