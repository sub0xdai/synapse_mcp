---
mcp: synapse
type: rule
title: "Test Commit Rule"
tags: ["test", "commit", "automation"]
---

# Test Commit Rule TC-001

## Purpose
This rule tests the automated git pre-commit hook integration.

## Requirements
- Pre-commit hooks must run automatically on commit
- Only files with `mcp: synapse` marker should be indexed
- Performance must remain under 500ms

## Status
Testing ⏳