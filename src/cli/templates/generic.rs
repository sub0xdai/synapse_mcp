use anyhow::Result;
use std::path::Path;
use super::{write_template_file, replace_placeholders};

pub async fn deploy_templates(project_name: &str) -> Result<()> {
    // Create coding standards template
    let coding_standards = r#"---
mcp: synapse
type: rule
title: "{{PROJECT_NAME}} Coding Standards"
tags: ["standards", "style", "quality"]
---

# {{PROJECT_NAME}} Coding Standards

## Overview
This document defines the coding standards and style guidelines for the {{PROJECT_NAME}} project.

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
"#;

    let path = Path::new(".synapse/rules/coding_standards.md");
    write_template_file(path, &replace_placeholders(coding_standards, project_name)).await?;
    
    // Create architecture overview template
    let architecture = r#"---
mcp: synapse
type: architecture
title: "{{PROJECT_NAME}} Architecture Overview"
tags: ["architecture", "overview", "design"]
---

# {{PROJECT_NAME}} Architecture Overview

## System Overview
Brief description of what {{PROJECT_NAME}} does and its main purpose.

## High-Level Architecture

### Components
- **Component A**: Description of main component
- **Component B**: Description of supporting component
- **Component C**: Description of integration component

### Data Flow
1. Input → Processing → Output
2. Describe the main data flow paths
3. Highlight important transformation points

## Technology Stack

### Core Technologies
- Language/Framework: [Specify main technology]
- Database: [Database technology]
- API: [API technology/protocol]

### Supporting Tools
- Build System: [Build tool]
- Testing: [Testing frameworks]
- Deployment: [Deployment method]

## Security Considerations
- Authentication and authorization approach
- Data encryption and protection
- Input validation strategy
- API security measures

## Performance Characteristics
- Expected load and scalability requirements
- Performance benchmarks and targets
- Monitoring and observability strategy

## Deployment Architecture
- Production environment setup
- Staging and development environments
- CI/CD pipeline overview
"#;

    let path = Path::new(".synapse/architecture/overview.md");
    write_template_file(path, &replace_placeholders(architecture, project_name)).await?;
    
    // Create testing strategy template
    let testing = r#"---
mcp: synapse
type: rule
title: "{{PROJECT_NAME}} Testing Strategy"
tags: ["testing", "quality", "automation"]
---

# {{PROJECT_NAME}} Testing Strategy

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
"#;

    let path = Path::new(".synapse/rules/testing_strategy.md");
    write_template_file(path, &replace_placeholders(testing, project_name)).await?;
    
    // Create decision template
    let decision = r#"---
mcp: synapse
type: decision
title: "ADR-001: Architecture Decision Template"
tags: ["adr", "template", "decision"]
---

# ADR-001: Architecture Decision Template

## Status
Accepted | Rejected | Superseded by [ADR-###]

## Context
Describe the situation that requires a decision. Include:
- What is the issue that we're seeing?
- What are the constraints?
- What are the requirements?

## Decision
Describe the change that we're making. Be specific about:
- What we will do
- What we won't do
- Why this approach over alternatives

## Alternatives Considered

### Alternative 1: [Name]
- **Pros**: List benefits
- **Cons**: List drawbacks
- **Decision**: Why accepted/rejected

### Alternative 2: [Name]
- **Pros**: List benefits
- **Cons**: List drawbacks
- **Decision**: Why accepted/rejected

## Consequences
Describe the expected outcomes of this decision:

### Positive
- Benefit 1
- Benefit 2

### Negative
- Trade-off 1
- Trade-off 2

### Risks
- Risk 1 and mitigation strategy
- Risk 2 and mitigation strategy

## Implementation Notes
- Timeline for implementation
- Required resources
- Migration strategy (if applicable)
- Monitoring and success metrics
"#;

    let path = Path::new(".synapse/decisions/adr_template.md");
    write_template_file(path, &replace_placeholders(decision, project_name)).await?;
    
    Ok(())
}