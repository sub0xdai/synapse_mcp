---
mcp: synapse
type: rule
title: "synapse-project Rust Coding Standards"
tags: ["rust", "standards", "style", "clippy"]
---

# synapse-project Rust Coding Standards

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
