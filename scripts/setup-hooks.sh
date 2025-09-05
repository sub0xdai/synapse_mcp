#!/bin/bash

# Synapse MCP Hook Setup Script
# Automated setup for both pre-commit and Claude context hooks

set -e

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPTS_DIR="$PROJECT_ROOT/scripts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

log_success() {
    echo -e "${GREEN}✅${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

log_error() {
    echo -e "${RED}❌${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        log_error "Not in a git repository"
        exit 1
    fi
    
    # Check if Synapse MCP is built
    if [ ! -f "$PROJECT_ROOT/target/release/synapse" ] && [ ! -f "$PROJECT_ROOT/target/debug/synapse" ]; then
        log_warning "Synapse MCP binary not found"
        log_info "Building Synapse MCP..."
        
        cd "$PROJECT_ROOT"
        if cargo build --release; then
            log_success "Synapse MCP built successfully"
        else
            log_error "Failed to build Synapse MCP"
            exit 1
        fi
    else
        log_success "Synapse MCP binary found"
    fi
    
    # Check for required tools
    if ! command -v jq &> /dev/null; then
        log_warning "jq not found - required for advanced context features"
        log_info "Install with: sudo apt install jq (Ubuntu/Debian) or brew install jq (macOS)"
    fi
    
    # Check if pre-commit is installed
    if ! command -v pre-commit &> /dev/null; then
        log_warning "pre-commit not found"
        if command -v uv &> /dev/null; then
            log_info "Installing pre-commit via uv..."
            uv tool install pre-commit
        elif command -v pip &> /dev/null; then
            log_info "Installing pre-commit via pip..."
            pip install pre-commit
        else
            log_error "Please install pre-commit manually"
            exit 1
        fi
    fi
}

# Setup pre-commit hook
setup_precommit_hook() {
    log_info "Setting up pre-commit hook..."
    
    # Create .pre-commit-config.yaml if it doesn't exist
    if [ ! -f ".pre-commit-config.yaml" ]; then
        log_info "Creating .pre-commit-config.yaml..."
        cat > .pre-commit-config.yaml << 'EOF'
repos:
  # Synapse MCP rule enforcement
  - repo: local
    hooks:
      - id: synapse-rules
        name: Synapse Rule Enforcement
        entry: bash
        language: system
        # Run on all file types
        files: '.*'
        # Use the pre-commit hook script
        args: ['-c', 'exec scripts/pre-commit-hook.sh', '--']
        pass_filenames: false
        stages: [pre-commit]
EOF
        log_success "Created .pre-commit-config.yaml"
    else
        log_info ".pre-commit-config.yaml already exists"
        
        # Check if Synapse hook is already configured
        if ! grep -q "synapse-rules" .pre-commit-config.yaml; then
            log_warning "Adding Synapse hook to existing .pre-commit-config.yaml"
            cat >> .pre-commit-config.yaml << 'EOF'

  # Synapse MCP rule enforcement
  - repo: local
    hooks:
      - id: synapse-rules
        name: Synapse Rule Enforcement
        entry: bash
        language: system
        files: '.*'
        args: ['-c', 'exec scripts/pre-commit-hook.sh', '--']
        pass_filenames: false
        stages: [pre-commit]
EOF
            log_success "Added Synapse hook to .pre-commit-config.yaml"
        else
            log_success "Synapse hook already configured in .pre-commit-config.yaml"
        fi
    fi
    
    # Install pre-commit hooks
    if pre-commit install; then
        log_success "Pre-commit hooks installed"
    else
        log_error "Failed to install pre-commit hooks"
        exit 1
    fi
    
    # Make hook script executable
    chmod +x "$SCRIPTS_DIR/pre-commit-hook.sh"
    log_success "Pre-commit hook setup complete"
}

# Setup Claude context hook
setup_claude_hook() {
    log_info "Setting up Claude context hook..."
    
    # Make Claude hook script executable
    chmod +x "$SCRIPTS_DIR/claude-context-hook.sh"
    
    # Add to PATH in common shell configuration files
    local synapse_path_export="export PATH=\"$PROJECT_ROOT/target/release:\$PATH\""
    local synapse_alias="alias claude-context=\"$SCRIPTS_DIR/claude-context-hook.sh\""
    
    # Add to .bashrc if it exists
    if [ -f ~/.bashrc ]; then
        if ! grep -q "synapse.*target/release" ~/.bashrc; then
            echo "" >> ~/.bashrc
            echo "# Synapse MCP" >> ~/.bashrc
            echo "$synapse_path_export" >> ~/.bashrc
            echo "$synapse_alias" >> ~/.bashrc
            log_success "Added Synapse MCP to ~/.bashrc"
        fi
    fi
    
    # Add to .zshrc if it exists
    if [ -f ~/.zshrc ]; then
        if ! grep -q "synapse.*target/release" ~/.zshrc; then
            echo "" >> ~/.zshrc
            echo "# Synapse MCP" >> ~/.zshrc
            echo "$synapse_path_export" >> ~/.zshrc
            echo "$synapse_alias" >> ~/.zshrc
            log_success "Added Synapse MCP to ~/.zshrc"
        fi
    fi
    
    log_success "Claude context hook setup complete"
}

# Setup environment
setup_environment() {
    log_info "Setting up environment..."
    
    # Add context file to .gitignore if not present
    if [ -f ".gitignore" ]; then
        if ! grep -q ".synapse_context" .gitignore; then
            echo "" >> .gitignore
            echo "# Synapse MCP context file" >> .gitignore
            echo ".synapse_context" >> .gitignore
            log_success "Added .synapse_context to .gitignore"
        fi
    else
        cat > .gitignore << 'EOF'
# Synapse MCP context file
.synapse_context
EOF
        log_success "Created .gitignore with .synapse_context"
    fi
    
    # Create example .env if not exists
    if [ ! -f ".env" ] && [ -f ".env.example" ]; then
        cp .env.example .env
        log_success "Created .env from .env.example"
    fi
}

# Test the setup
test_setup() {
    log_info "Testing setup..."
    
    # Test pre-commit hook
    if pre-commit run --all-files --verbose; then
        log_success "Pre-commit hooks test passed"
    else
        log_warning "Pre-commit hooks test had issues (this may be expected if no rule files exist)"
    fi
    
    # Test Claude context generation
    if "$SCRIPTS_DIR/claude-context-hook.sh" context all markdown; then
        log_success "Claude context generation test passed"
        if [ -f ".synapse_context" ]; then
            local lines=$(wc -l < .synapse_context)
            log_info "Generated context file with $lines lines"
        fi
    else
        log_warning "Claude context generation test failed (this may be expected if no rule files exist)"
    fi
}

# Main setup function
main() {
    local action="${1:-setup}"
    
    echo "🔧 Synapse MCP Hook Setup"
    echo "========================="
    echo
    
    case $action in
        "setup")
            check_prerequisites
            echo
            setup_environment
            echo
            setup_precommit_hook
            echo
            setup_claude_hook
            echo
            test_setup
            echo
            log_success "Synapse MCP hooks setup complete!"
            echo
            echo "📋 Next steps:"
            echo "  1. Create .synapse.md rule files in your project"
            echo "  2. Commit changes - the pre-commit hook will enforce rules"
            echo "  3. Use 'claude-context context' to generate AI context"
            echo
            echo "💡 Usage examples:"
            echo "  # Generate context for Claude"
            echo "  claude-context context all markdown"
            echo
            echo "  # Check MCP server status"
            echo "  claude-context status"
            echo
            echo "  # Test pre-commit hooks manually"
            echo "  pre-commit run --all-files"
            echo
            echo "🎉 Your project now has intelligent rule enforcement!"
            ;;
        "test")
            test_setup
            ;;
        "clean")
            log_info "Cleaning up hooks and temporary files..."
            pre-commit uninstall 2>/dev/null || true
            rm -f .synapse_context
            pkill -f "synapse.*serve" 2>/dev/null || true
            log_success "Cleanup complete"
            ;;
        "status")
            log_info "Checking setup status..."
            
            if [ -f ".pre-commit-config.yaml" ]; then
                log_success "Pre-commit configuration exists"
            else
                log_warning "Pre-commit configuration not found"
            fi
            
            if pre-commit --version >/dev/null 2>&1; then
                log_success "Pre-commit is installed"
            else
                log_warning "Pre-commit is not installed"
            fi
            
            if [ -x "$SCRIPTS_DIR/claude-context-hook.sh" ]; then
                log_success "Claude context hook is executable"
            else
                log_warning "Claude context hook is not executable"
            fi
            
            if [ -f "$PROJECT_ROOT/target/release/synapse" ]; then
                log_success "Synapse binary found (release)"
            elif [ -f "$PROJECT_ROOT/target/debug/synapse" ]; then
                log_success "Synapse binary found (debug)"
            else
                log_warning "Synapse binary not found"
            fi
            ;;
        *)
            echo "Usage: $0 {setup|test|clean|status}"
            echo
            echo "Commands:"
            echo "  setup  - Full setup of automation hooks (default)"
            echo "  test   - Test the hook setup"
            echo "  clean  - Remove hooks and temporary files"
            echo "  status - Check current setup status"
            exit 1
            ;;
    esac
}

# Handle command line arguments
main "$@"