#!/bin/bash

# Setup script for Synapse MCP automation hooks
# This script sets up the complete automation pipeline

set -e

echo "🔧 Synapse MCP Hook Setup"
echo "========================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✅${NC} $1"
}

log_error() {
    echo -e "${RED}❌${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ] || [ ! -f "src/main.rs" ]; then
        log_error "Must be run from the synapse_mcp project root directory"
        exit 1
    fi
    
    # Check if uv is installed
    if ! command -v uv &> /dev/null; then
        log_error "uv is required but not installed"
        exit 1
    fi
    
    # Check if pre-commit is installed
    if ! command -v pre-commit &> /dev/null; then
        log_info "Installing pre-commit..."
        uv tool install pre-commit
    fi
    
    # Check if jq is installed (needed for Claude hook)
    if ! command -v jq &> /dev/null; then
        log_error "jq is required for Claude hook functionality"
        log_info "Install with: sudo apt install jq (Ubuntu/Debian) or brew install jq (macOS)"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Check Neo4j
check_neo4j() {
    log_info "Checking Neo4j availability..."
    
    if docker ps | grep -q neo4j; then
        log_success "Neo4j container is running"
    else
        log_info "Starting Neo4j container..."
        if [ -f "docker-compose.yml" ]; then
            docker-compose up -d neo4j
            sleep 5
            if docker ps | grep -q neo4j; then
                log_success "Neo4j container started"
            else
                log_error "Failed to start Neo4j container"
                exit 1
            fi
        else
            log_error "Neo4j is not running and no docker-compose.yml found"
            exit 1
        fi
    fi
}

# Setup environment
setup_environment() {
    log_info "Setting up environment..."
    
    if [ ! -f ".env" ]; then
        if [ -f ".env.example" ]; then
            cp .env.example .env
            log_success "Created .env file from .env.example"
        else
            log_error ".env.example not found"
            exit 1
        fi
    fi
    
    # Add context file to .gitignore if not present
    if [ -f ".gitignore" ]; then
        if ! grep -q ".synapse_context" .gitignore; then
            echo "" >> .gitignore
            echo "# Synapse MCP context file" >> .gitignore
            echo ".synapse_context" >> .gitignore
            log_success "Added .synapse_context to .gitignore"
        fi
    fi
}

# Setup git hooks
setup_git_hooks() {
    log_info "Setting up git pre-commit hooks..."
    
    if [ ! -f ".pre-commit-config.yaml" ]; then
        log_error ".pre-commit-config.yaml not found"
        exit 1
    fi
    
    pre-commit install
    log_success "Git pre-commit hooks installed"
}

# Test the pipeline
test_pipeline() {
    log_info "Testing the automation pipeline..."
    
    # Build the project
    log_info "Building project..."
    cargo build
    
    # Test the indexer with sample documents
    if [ -d "test_docs" ]; then
        log_info "Testing indexer with sample documents..."
        cargo run --bin indexer test_docs/*.md
        log_success "Indexer test passed"
    fi
    
    # Test Claude hook
    log_info "Testing Claude context hook..."
    ./claude-hook.sh status
    
    # Test context generation (this will start the server if needed)
    ./claude-hook.sh context
    if [ -f ".synapse_context" ]; then
        log_success "Context file generated successfully"
        log_info "Context preview:"
        head -20 .synapse_context
    else
        log_error "Failed to generate context file"
        exit 1
    fi
    
    log_success "Pipeline test completed"
}

# Main setup function
main() {
    echo
    check_prerequisites
    echo
    check_neo4j
    echo
    setup_environment
    echo
    setup_git_hooks
    echo
    test_pipeline
    echo
    
    log_success "Synapse MCP hooks setup complete!"
    echo
    echo "📋 Next steps:"
    echo "  1. Edit files in your project with 'mcp: synapse' frontmatter"
    echo "  2. Commit changes - the indexer will run automatically"
    echo "  3. Use './claude-hook.sh context' to get project context for AI"
    echo
    echo "💡 Usage examples:"
    echo "  # Generate context for Claude"
    echo "  ./claude-hook.sh context"
    echo
    echo "  # Check MCP server status"
    echo "  ./claude-hook.sh status"
    echo
    echo "  # Test pre-commit hooks manually"
    echo "  pre-commit run --all-files"
    echo
    echo "🎉 Your project now has intelligent, automated memory!"
}

# Handle command line arguments
case "${1:-setup}" in
    "setup")
        main
        ;;
    "test")
        test_pipeline
        ;;
    "clean")
        log_info "Cleaning up hooks and temporary files..."
        pre-commit uninstall 2>/dev/null || true
        rm -f .synapse_context
        ./claude-hook.sh stop 2>/dev/null || true
        log_success "Cleanup complete"
        ;;
    *)
        echo "Usage: $0 {setup|test|clean}"
        echo
        echo "Commands:"
        echo "  setup - Full setup of automation hooks (default)"
        echo "  test  - Test the automation pipeline"
        echo "  clean - Remove hooks and temporary files"
        exit 1
        ;;
esac