use anyhow::Result;
use std::path::Path;
use super::{write_template_file, replace_placeholders};

pub async fn deploy_templates(project_name: &str) -> Result<()> {
    // First deploy generic templates
    super::generic::deploy_templates(project_name).await?;
    
    // Then add Rust-specific templates
    let rust_coding_standards = r#"---
mcp: synapse
type: rule
title: "{{PROJECT_NAME}} Rust Coding Standards"
tags: ["rust", "standards", "style", "clippy"]
---

# {{PROJECT_NAME}} Rust Coding Standards

## Code Formatting
- **Always** use `cargo fmt` before committing
- Line length: 100 characters maximum
- Use trailing commas in multi-line expressions

## Naming Conventions
- **Types**: `PascalCase` (structs, enums, traits)
- **Functions/Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Modules**: `snake_case`

## Error Handling
- Use `Result<T, E>` for recoverable errors
- Use `anyhow::Result<T>` for application errors
- Use `thiserror` for library errors
- Never `unwrap()` or `expect()` in production code except for:
  - Static data that is guaranteed valid
  - Test code
  - Early development prototypes (mark with TODO)

## Performance Guidelines
- Use `&str` instead of `String` when possible
- Prefer borrowing over cloning
- Use `Vec::with_capacity()` when size is known
- Profile before optimizing

## Testing
- Unit tests in same file using `#[cfg(test)]`
- Integration tests in `tests/` directory
- Use `#[test]` for simple tests
- Use `#[tokio::test]` for async tests
- Mock external dependencies

## Documentation
- All public items must have doc comments (`///`)
- Include examples in doc comments for public APIs
- Use `#[doc(hidden)]` for internal public items

## Cargo Dependencies
- Minimize dependencies
- Prefer standard library when possible
- Pin major versions in `Cargo.toml`
- Regular dependency audit with `cargo audit`

## Linting
- All code must pass `cargo clippy -- -D warnings`
- Use `#[allow(clippy::...)]` sparingly with comments
- Run `cargo check` before committing
"#;

    let path = Path::new(".synapse/rules/rust_standards.md");
    write_template_file(path, &replace_placeholders(rust_coding_standards, project_name)).await?;
    
    let performance_guidelines = r#"---
mcp: synapse
type: rule
title: "{{PROJECT_NAME}} Performance Guidelines"
tags: ["performance", "rust", "optimization", "benchmarks"]
---

# {{PROJECT_NAME}} Performance Guidelines

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
"#;

    let path = Path::new(".synapse/rules/performance_guidelines.md");
    write_template_file(path, &replace_placeholders(performance_guidelines, project_name)).await?;

    let security_guidelines = r#"---
mcp: synapse
type: rule
title: "{{PROJECT_NAME}} Security Guidelines"
tags: ["security", "rust", "safety", "validation"]
---

# {{PROJECT_NAME}} Security Guidelines

## Input Validation
- Validate all external input (API, file, database)
- Use strong typing to prevent invalid states
- Sanitize data before database queries
- Use prepared statements for SQL queries

## Memory Safety
- Avoid `unsafe` blocks unless absolutely necessary
- When using `unsafe`, document safety invariants
- Use `cargo miri` for testing unsafe code
- Prefer safe alternatives (e.g., `Vec` over raw pointers)

## Authentication & Authorization
- Use JWT tokens with reasonable expiration
- Implement rate limiting on public endpoints
- Log security events for auditing
- Use HTTPS in production

## Data Protection
- Hash passwords with `argon2`
- Encrypt sensitive data at rest
- Use secure random number generation
- Implement proper session management

## Error Handling Security
- Never leak sensitive information in error messages
- Log security-relevant errors
- Use generic error messages for public APIs
- Implement proper error propagation

## Dependencies
- Regular security audits with `cargo audit`
- Keep dependencies up to date
- Review security advisories
- Minimize dependency surface area

## Logging
- Never log sensitive data (passwords, tokens, PII)
- Use structured logging for security events
- Implement log rotation and retention
- Monitor for suspicious patterns
"#;

    let path = Path::new(".synapse/rules/security_guidelines.md");
    write_template_file(path, &replace_placeholders(security_guidelines, project_name)).await?;

    Ok(())
}