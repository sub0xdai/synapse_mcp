---
mcp: synapse
type: rule
title: "synapse-project Testing Strategy"
tags: ["testing", "quality", "automation"]
---

# synapse-project Testing Strategy

## Testing Philosophy
Our testing approach focuses on confidence, speed, and maintainability.

## Test Types

### Unit Tests
- Test individual functions and classes in isolation
- Fast execution (< 10ms per test)
- High coverage of business logic
- Mock external dependencies

### Integration Tests
- Test component interactions
- Use real databases/services where practical
- Focus on critical user workflows
- Moderate execution time (< 1s per test)

### End-to-End Tests
- Test complete user scenarios
- Run against staging environment
- Limited number focusing on critical paths
- Acceptable longer execution time (< 30s per test)

## Test Requirements
- All new features must include tests
- Bug fixes must include regression tests
- Public APIs must have comprehensive test coverage
- Performance-critical code needs benchmark tests

## Test Data Management
- Use factories/builders for test data creation
- Clean up test data after test runs
- Use realistic but anonymized data
- Maintain separate test databases

## Continuous Integration
- Tests run automatically on every commit
- Failed tests block merging
- Test results are visible to all team members
- Flaky tests are addressed immediately

## Performance Testing
- Benchmark critical operations
- Load testing for high-traffic endpoints
- Memory and resource usage monitoring
- Regular performance regression testing
