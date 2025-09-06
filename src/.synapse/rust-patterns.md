---
mcp: synapse
type: rule
title: "Rust Patterns"
tags: ["rust", "patterns"]
---

# Rust Patterns

FORBIDDEN: `println!` - Use log macros (info!, warn!, error!) instead
REQUIRED: `#[derive(Debug)]` - All public structs should derive Debug
STANDARD: `snake_case` - Use snake_case for variables and functions
CONVENTION: `Result<T>` - Functions that can fail should return Result