
# Synapse

A CLI tool that builds an AI-readable knowledge base from your project's documentation to provide context for coding assistants and enforce local standards.

## Core Features

  * **AI Context Generation**: Creates context files from your documentation for AI assistants like Claude.
  * **Rule Enforcement**: Validates code against project-specific standards using a built-in engine.
  * **Project Scaffolding**: Initializes a `.synapse/` directory with templates for rules, architecture, and decisions.
  * **Git Integration**: Automates rule checking and context updates via pre-commit hooks.

-----

## Getting Started

### 1\. Installation

First, clone the repository and build the binary using Cargo.

```bash
git clone <your-synapse-repo>
cd synapse
cargo build --release

# Optional: Add the binary to your system's PATH
export PATH="$PWD/target/release:$PATH"
```

### 2\. Initialize Your Project

Navigate to your target project's directory and run the `init` command. This creates a `.synapse/` directory with documentation templates.

```bash
# In your project's root directory
synapse init --template=rust
```

Available templates include `rust`, `python`, `typescript`, and `generic`.

### 3\. Populate Your Documentation

Edit the generated Markdown templates inside the `.synapse/` directory with your project's specific standards, architecture, and decisions.

```bash
$EDITOR .synapse/rules/coding_standards.md
$EDITOR .synapse/architecture/overview.md
```

### 4\. Generate AI Context

Run the `context` command to generate the `.synapse_context` file that your AI assistant will use.

```bash
synapse context --scope=all
```

### 5\. (Optional) Setup Git Hooks

Automate rule enforcement on commit and keep your AI context up-to-date.

```bash
./scripts/setup-hooks.sh
```

-----

## Usage

Here are the main commands for operating Synapse.

| Command | Description |
| :--- | :--- |
| `synapse init` | Initializes a `.synapse/` workspace in the current directory. |
| `synapse context` | Generates an AI context file from documentation. |
| `synapse check` | Checks specified files against the defined rules. |
| `synapse query` | Asks a question of the project's knowledge base. |
| `synapse serve` | Starts the MCP server with API endpoints for enforcement. |
| `synapse status` | Displays the status of the workspace and its components. |

-----

## Configuration

Synapse is configured through two primary methods:

1.  **Documents**: All `.md` files in the `.synapse/` directory are parsed. They must contain YAML frontmatter with `mcp: synapse` and a `type` field (`rule`, `architecture`, etc.).
2.  **Environment Variables**: Optional variables for advanced configuration.
      * `NEO4J_URI`: URI for the Neo4j database connection.
      * `SYNAPSE_VERBOSE`: Set to `true` for detailed logging.
      * `SYNAPSE_CONTEXT_FILE`: The default output name for the context file.

-----

## Architecture

Synapse works by parsing Markdown documents into an in-memory **RuleGraph**. This graph understands the relationships and inheritance between your project's standards.

The **PatternEnforcer** engine then uses this graph for two purposes:

  * **Write Path (Enforcement)**: A `git commit` triggers a hook that uses the engine to validate code changes.
  * **Read Path (AI Context)**: A context hook or manual command uses the engine to generate a summary of relevant rules for an AI assistant.

For a more detailed breakdown, see `architecture.md`.
