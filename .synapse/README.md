# Synapse AI Memory Workspace

This directory contains AI-readable documentation that helps build context for AI coding assistants.

## Structure

- `rules/` - Development rules and coding standards
- `architecture/` - High-level architecture documentation
- `decisions/` - Architecture decision records (ADRs)
- `components/` - Component specifications and documentation
- `templates/` - Document templates for consistency

## Usage

1. Fill out the template files in each directory
2. Create new documents using the templates
3. Use `synapse context` to generate AI context
4. All documents are automatically indexed on git commit

## Document Format

All documents should include YAML frontmatter:

```yaml
---
mcp: synapse
type: rule|architecture|decision|component
title: "Document Title"
tags: ["tag1", "tag2"]
---
```

Only documents with `mcp: synapse` will be indexed.
