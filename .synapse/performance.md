---
mcp: synapse
type: rule
title: "Performance Rules"
tags: ["performance", "optimization"]
---

# Performance Rules

FORBIDDEN: `unwrap()` - Use proper error handling instead of unwrap()
FORBIDDEN: `clone()` - Avoid unnecessary clones in hot paths
REQUIRED: `async` - Functions that do I/O must be async
STANDARD: `Vec::with_capacity` - Pre-allocate vectors when size is known
CONVENTION: `_fast` - Performance-critical functions should end with _fast