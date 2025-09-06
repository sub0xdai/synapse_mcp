---
mcp: synapse
type: rule
title: "synapse-project Security Guidelines"
tags: ["security", "rust", "safety", "validation"]
---

# synapse-project Security Guidelines

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
