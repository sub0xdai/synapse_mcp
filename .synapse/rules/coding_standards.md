---
mcp: synapse
type: rule
title: "synapse-project Coding Standards"
tags: ["standards", "style", "quality"]
---

# synapse-project Coding Standards

## Overview
This document defines the coding standards and style guidelines for the synapse-project project.

## General Principles
- **Consistency**: Follow established patterns within the codebase
- **Readability**: Write code that tells a story
- **Maintainability**: Make changes easy for future developers
- **Testing**: All features must have appropriate test coverage

## Code Style
- Use descriptive variable and function names
- Keep functions small and focused (single responsibility)
- Comment complex logic and business rules
- Remove dead code and unused imports

## Documentation
- Update documentation when changing functionality
- Include examples in API documentation
- Maintain up-to-date README files

## Review Process
- All code changes require peer review
- Address review comments before merging
- Test changes in staging environment

## Quality Gates
- Code must pass all linting checks
- All tests must pass before merging
- Security scans must pass
- Performance benchmarks must be within acceptable thresholds
