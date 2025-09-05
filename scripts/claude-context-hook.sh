#!/bin/bash

# Synapse MCP Claude Context Hook
# This script generates AI context for Claude Code sessions

set -e

# Configuration
SYNAPSE_BINARY="${SYNAPSE_BINARY:-synapse}"
SYNAPSE_MCP_URL="${SYNAPSE_MCP_URL:-http://localhost:8080}"
CONTEXT_FILE="${SYNAPSE_CONTEXT_FILE:-.synapse_context}"
SYNAPSE_VERBOSE="${SYNAPSE_VERBOSE:-false}"

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

# Function to check if MCP server is running
check_server() {
    if curl -s "$SYNAPSE_MCP_URL/health" >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to start MCP server if not running
start_server_if_needed() {
    if ! check_server; then
        log_info "Starting Synapse MCP server..."
        
        # Check if synapse binary exists
        if ! command -v $SYNAPSE_BINARY &> /dev/null; then
            log_error "Synapse binary '$SYNAPSE_BINARY' not found"
            log_info "Please build Synapse MCP: cargo build --release"
            return 1
        fi
        
        # Kill any existing background server
        pkill -f "$SYNAPSE_BINARY.*serve" 2>/dev/null || true
        
        # Start server in background with rule enforcement
        nohup $SYNAPSE_BINARY serve --enable-enforcer --port 8080 >/dev/null 2>&1 &
        
        # Wait for server to be ready
        local retries=10
        while ! check_server && [ $retries -gt 0 ]; do
            sleep 1
            retries=$((retries - 1))
        done
        
        if check_server; then
            log_success "MCP server is ready"
        else
            log_error "Failed to start MCP server"
            return 1
        fi
    else
        if [ "$SYNAPSE_VERBOSE" = "true" ]; then
            log_success "MCP server is already running"
        fi
    fi
}

# Function to generate context using CLI (fallback)
generate_context_cli() {
    local scope="${1:-all}"
    local format="${2:-markdown}"
    
    if ! command -v $SYNAPSE_BINARY &> /dev/null; then
        log_error "Synapse binary not found"
        return 1
    fi
    
    local cmd="$SYNAPSE_BINARY context --scope=$scope --format=$format --output=$CONTEXT_FILE"
    
    if [ "$SYNAPSE_VERBOSE" = "true" ]; then
        log_info "Running: $cmd"
        cmd="$cmd --verbose"
    fi
    
    if eval $cmd; then
        log_success "Context generated using CLI"
        return 0
    else
        log_error "Failed to generate context using CLI"
        return 1
    fi
}

# Function to generate context via API
generate_context_api() {
    local path="${1:-$(pwd)}"
    local format="${2:-markdown}"
    
    local request_data=$(cat <<EOF
{
    "path": "$path",
    "format": "$format"
}
EOF
)
    
    if [ "$SYNAPSE_VERBOSE" = "true" ]; then
        log_info "Making API request to $SYNAPSE_MCP_URL/enforce/context"
    fi
    
    local response=$(curl -s -X POST "$SYNAPSE_MCP_URL/enforce/context" \
        -H "Content-Type: application/json" \
        -d "$request_data")
    
    if [ $? -eq 0 ]; then
        local success=$(echo "$response" | jq -r '.success // false' 2>/dev/null)
        if [ "$success" = "true" ]; then
            local context=$(echo "$response" | jq -r '.context // ""' 2>/dev/null)
            if [ -n "$context" ] && [ "$context" != "null" ]; then
                echo "$context" > "$CONTEXT_FILE"
                log_success "Context generated via API"
                return 0
            fi
        fi
    fi
    
    log_warning "API context generation failed, falling back to CLI"
    return 1
}

# Main function
main() {
    local action="${1:-context}"
    local scope_or_path="${2:-all}"
    local format="${3:-markdown}"
    
    case $action in
        "start")
            start_server_if_needed
            ;;
        "context")
            log_info "🧠 Generating AI context for Claude Code..."
            
            # Try API first (with PatternEnforcer), then fall back to CLI
            if start_server_if_needed && generate_context_api "$scope_or_path" "$format"; then
                :  # Success via API
            elif generate_context_cli "$scope_or_path" "$format"; then
                :  # Success via CLI
            else
                log_error "Failed to generate context"
                exit 1
            fi
            
            if [ -f "$CONTEXT_FILE" ]; then
                local lines=$(wc -l < "$CONTEXT_FILE")
                log_success "Context saved to $CONTEXT_FILE ($lines lines)"
                
                if [ "$SYNAPSE_VERBOSE" = "true" ]; then
                    log_info "Context preview:"
                    head -20 "$CONTEXT_FILE"
                fi
            else
                log_error "Context file was not created"
                exit 1
            fi
            ;;
        "stop")
            pkill -f "$SYNAPSE_BINARY.*serve" 2>/dev/null || true
            log_success "MCP server stopped"
            ;;
        "status")
            if check_server; then
                log_success "MCP server is running"
                if [ "$SYNAPSE_VERBOSE" = "true" ]; then
                    curl -s "$SYNAPSE_MCP_URL/health" | jq '.' 2>/dev/null || true
                fi
            else
                log_warning "MCP server is not running"
                exit 1
            fi
            ;;
        "clean")
            pkill -f "$SYNAPSE_BINARY.*serve" 2>/dev/null || true
            rm -f "$CONTEXT_FILE"
            log_success "Cleaned up server and context file"
            ;;
        *)
            echo "Usage: $0 {start|context|stop|status|clean} [scope/path] [format]"
            echo ""
            echo "Commands:"
            echo "  start                      - Start MCP server if not running"
            echo "  context [scope] [format]   - Generate context file for Claude (default: all, markdown)"
            echo "  stop                       - Stop MCP server"
            echo "  status                     - Check if server is running"
            echo "  clean                      - Stop server and remove context file"
            echo ""
            echo "Scopes: all, rules, architecture, decisions, test, api"
            echo "Formats: markdown, json, plain"
            echo ""
            echo "Environment Variables:"
            echo "  SYNAPSE_BINARY       - Path to synapse binary (default: synapse)"
            echo "  SYNAPSE_MCP_URL      - MCP server URL (default: http://localhost:8080)"
            echo "  SYNAPSE_CONTEXT_FILE - Context file path (default: .synapse_context)"
            echo "  SYNAPSE_VERBOSE      - Enable verbose output (default: false)"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"