---
mcp: synapse
type: rule
title: "Project-Wide Development Rules"
tags: ["global", "standards", "quality"]
---

# Project-Wide Development Rules

These rules apply to all files in the project unless overridden by more specific rules.

## Code Quality Rules

FORBIDDEN: `TODO` - Convert TODO comments to proper issue tracking with GitHub issues or tickets
FORBIDDEN: `console.log` - Use structured logging framework instead of console.log
FORBIDDEN: `println!` - Use log crate (info!, warn!, error!) instead of println! for production code
FORBIDDEN: `unwrap()` - Use proper error handling with `?` operator or `expect()` with meaningful messages

## Required Standards

REQUIRED: `SPDX-License-Identifier` - All source files must include SPDX license identifier in header
REQUIRED: `#\[derive\(Debug\)\]` - All public structs should derive Debug for better error reporting

## Style Guidelines

STANDARD: `snake_case` - Use snake_case for Rust variables and functions per Rust conventions
STANDARD: `async` functions should complete within 500ms - Performance requirement for async operations
CONVENTION: Use descriptive variable names - Avoid single letter variables except for iterators
CONVENTION: Add doc comments to public APIs - Use `///` for public functions and structs

## Security Requirements

FORBIDDEN: `password` - Never hardcode passwords or secrets in source code
REQUIRED: `validate_input` - All user inputs must be validated and sanitized
STANDARD: `https://` - Use HTTPS for all external API calls