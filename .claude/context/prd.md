# Product Requirements Document: Synapse MCP

## 1. Vision

To create a "living" knowledge system that actively enforces a project's architectural rules and coding standards, guiding both human and AI developers to write compliant code in real-time.

## 2. Problem

Developers and AI assistants struggle to maintain architectural integrity and adhere to project-specific conventions in large, evolving codebases. This leads to inconsistent code, technical debt, and repeated manual corrections. There is no active, intelligent system to enforce these rules *as code is being written* or provide context *as it is being requested*.

## 3. Target User

Software developers using AI assistants (e.g., via Serena MCP) within their local development environment.

## 4. Core Features (User Stories)

- **As a developer**, I want to define project rules and standards in simple, distributed Markdown files (`.synapse.md`) so that rules live with the code they govern.

- **As a developer**, I want rules from parent directories to be automatically inherited by subdirectories, so I can define global standards and override them only when necessary for specific contexts.

- **As a developer**, I want the system to use a **Write Hook** (e.g., a pre-commit hook) to check my changes against the rules, preventing non-compliant code from being committed.

- **As a developer**, I want my AI assistant to use a **Pre-Write Hook** that validates my code in real-time, providing instant feedback, context, and automated corrections based on project-specific rules.

- **As a developer**, I want to configure different enforcement levels (e.g., `strict`, `standard`, `learning`) for different projects or teams to allow for progressive adoption and a smoother learning curve.

- **As a developer**, I want the system to build a knowledge graph of all rules, so it can understand and query complex relationships like inheritance, overrides, and enforcement patterns.

## 5. Success Metrics

- **Performance:** The pre-commit hook (Write Hook) must complete its check in under 500ms for an average-sized change.
- **AI Quality:** A measurable increase in the AI's ability to generate code that adheres to documented rules without manual prompting.
- **Developer Workflow:** The system provides clear, actionable feedback for violations, reducing the time spent on manual code reviews for style and architecture.
- **Adoption:** The tool is successfully integrated into a developer's daily workflow across multiple enforcement levels.