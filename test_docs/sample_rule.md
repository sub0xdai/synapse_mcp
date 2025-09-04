---
mcp: synapse
type: rule
title: "Performance Optimization Rule"
tags: ["performance", "optimization", "rust"]
---

# Performance Rule PR-001

## Overview
All async operations must complete within 500ms for optimal user experience.

## Implementation Guidelines
- Use connection pooling for database operations
- Implement proper error handling with timeouts
- Cache frequently accessed data structures

## Related Components
- [Component A] handles database connections
- [Architecture Decision ADR-001] defines the performance requirements