---
mcp: synapse
type: rule
title: "synapse-project Performance Guidelines"
tags: ["performance", "rust", "optimization", "benchmarks"]
---

# synapse-project Performance Guidelines

## Performance Requirements
- API responses: < 100ms for 95th percentile
- Database queries: < 50ms average
- Memory usage: < 512MB for typical workloads
- CPU usage: < 80% under normal load

## Optimization Strategy

### Memory Management
- Use `Box<T>` for large objects on heap
- Prefer `Rc<T>` over `Arc<T>` in single-threaded contexts
- Use `Cow<T>` for conditional cloning
- Avoid unnecessary `clone()` calls

### Async Performance
- Use `tokio::spawn()` for CPU-intensive tasks
- Batch database operations
- Use connection pooling
- Implement proper backpressure

### Collections
- Use `HashMap` for key-value lookups
- Use `BTreeMap` when ordering matters
- Use `Vec` for sequential access
- Use `VecDeque` for push/pop at both ends

## Benchmarking
- Use `criterion` crate for micro-benchmarks
- Benchmark critical functions regularly
- Include benchmarks in CI pipeline
- Profile with `cargo flamegraph`

## Monitoring
- Use structured logging with `tracing`
- Implement health check endpoints
- Monitor key metrics:
  - Request latency
  - Memory usage
  - CPU utilization
  - Database connection pool usage

## Database Performance
- Use prepared statements
- Implement proper indexing
- Use connection pooling (max 10 connections)
- Batch operations when possible
- Use `EXPLAIN ANALYZE` for query optimization
