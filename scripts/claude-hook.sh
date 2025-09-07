#!/bin/bash

# Claude Code Hook for Synapse MCP
# This script automatically injects project context from the knowledge graph

set -e

SERVER_URL="${SYNAPSE_MCP_URL:-http://localhost:8080}"
CONTEXT_FILE="${SYNAPSE_CONTEXT_FILE:-.synapse_context}"

# Function to check if MCP server is running
check_server() {
    if curl -s "$SERVER_URL/health" >/dev/null 2>&1; then
        return 0
    else
        return 1
    fi
}

# Function to start MCP server if not running
start_server_if_needed() {
    if ! check_server; then
        echo "🚀 Starting Synapse MCP server..."
        # Kill background server if it was previously running
        pkill -f "synapse_mcp server" 2>/dev/null || true
        
        # Start server in background
        nohup cargo run --bin synapse_mcp server --port 8080 >/dev/null 2>&1 &
        
        # Wait for server to be ready
        local retries=10
        while ! check_server && [ $retries -gt 0 ]; do
            sleep 1
            retries=$((retries - 1))
        done
        
        if check_server; then
            echo "✅ Synapse MCP server is ready"
        else
            echo "❌ Failed to start MCP server"
            return 1
        fi
    else
        echo "✅ Synapse MCP server is already running"
    fi
}

# Function to get context from MCP server
get_context() {
    local query_type="${1:-rules}"
    local context=""
    
    case $query_type in
        "rules")
            # Get all rules
            context=$(curl -s "$SERVER_URL/nodes/rule" | jq -r '
                if .success then
                    "# Project Rules\n" +
                    (.nodes[] | "- **\(.label)**: \(.content | split("\n")[0] | sub("^# "; ""))")
                else
                    ""
                end
            ' 2>/dev/null)
            ;;
        "architecture")
            # Get architecture decisions
            context=$(curl -s "$SERVER_URL/nodes/architecture" | jq -r '
                if .success then
                    "# Architecture Decisions\n" +
                    (.nodes[] | "- **\(.label)**: \(.content | split("\n")[0] | sub("^# "; ""))")
                else
                    ""
                end
            ' 2>/dev/null)
            ;;
        "decisions")
            # Get all decisions
            context=$(curl -s "$SERVER_URL/nodes/decision" | jq -r '
                if .success then
                    "# Project Decisions\n" +
                    (.nodes[] | "- **\(.label)**: \(.content | split("\n")[0] | sub("^# "; ""))")
                else
                    ""
                end
            ' 2>/dev/null)
            ;;
        *)
            # Natural language query
            local query_data=$(jq -n --arg query "$query_type" '{query: $query}')
            context=$(curl -s -X POST "$SERVER_URL/query" \
                -H "Content-Type: application/json" \
                -d "$query_data" | jq -r '
                if .success then
                    "# Context from Query: \(.result)"
                else
                    ""
                end
            ' 2>/dev/null)
            ;;
    esac
    
    echo "$context"
}

# Function to create context file for Claude
create_context_file() {
    local file_pattern="${1:-}"
    local context_types="rules architecture decisions"
    
    echo "🧠 Gathering project context from Synapse MCP..."
    
    {
        echo "# SYNAPSE MCP CONTEXT"
        echo "# Auto-generated project context from knowledge graph"
        echo "# Generated at: $(date)"
        echo ""
        
        for type in $context_types; do
            local ctx=$(get_context "$type")
            if [ -n "$ctx" ] && [ "$ctx" != "null" ] && [ "$ctx" != "" ]; then
                echo "$ctx"
                echo ""
            fi
        done
        
        # If we have a file pattern, try a natural language query
        if [ -n "$file_pattern" ]; then
            echo "# Context for: $file_pattern"
            local query_ctx=$(get_context "context for $file_pattern")
            if [ -n "$query_ctx" ] && [ "$query_ctx" != "null" ] && [ "$query_ctx" != "" ]; then
                echo "$query_ctx"
                echo ""
            fi
        fi
        
    } > "$CONTEXT_FILE"
    
    echo "📝 Context saved to $CONTEXT_FILE"
}

# Main function
main() {
    local action="${1:-context}"
    local file_pattern="${2:-}"
    
    case $action in
        "start")
            start_server_if_needed
            ;;
        "context")
            start_server_if_needed
            create_context_file "$file_pattern"
            ;;
        "stop")
            pkill -f "synapse_mcp server" 2>/dev/null || true
            echo "🛑 Stopped Synapse MCP server"
            ;;
        "status")
            if check_server; then
                echo "✅ Synapse MCP server is running"
            else
                echo "❌ Synapse MCP server is not running"
            fi
            ;;
        *)
            echo "Usage: $0 {start|context|stop|status} [file_pattern]"
            echo ""
            echo "Commands:"
            echo "  start              - Start MCP server if not running"
            echo "  context [pattern]  - Generate context file for Claude"
            echo "  stop               - Stop MCP server"
            echo "  status             - Check if server is running"
            echo ""
            echo "Environment Variables:"
            echo "  SYNAPSE_MCP_URL      - MCP server URL (default: http://localhost:8080)"
            echo "  SYNAPSE_CONTEXT_FILE - Context file path (default: .synapse_context)"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@"