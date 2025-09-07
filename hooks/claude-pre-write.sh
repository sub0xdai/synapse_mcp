#!/bin/bash

# Claude Code Pre-Write Hook for Synapse MCP
# This script validates content before Claude Code writes to files
# Usage: claude-pre-write.sh <file_path> <content>

set -e

# Configuration
SYNAPSE_MCP_URL="${SYNAPSE_MCP_URL:-http://localhost:8080}"
SYNAPSE_AUTH_TOKEN="${SYNAPSE_AUTH_TOKEN:-}"
TEMP_FILE=$(mktemp /tmp/synapse_content.XXXXXX)
RESPONSE_FILE=$(mktemp /tmp/synapse_response.XXXXXX)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}ℹ${NC} $1" >&2
}

log_success() {
    echo -e "${GREEN}✅${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}⚠️${NC} $1" >&2
}

log_error() {
    echo -e "${RED}❌${NC} $1" >&2
}

# Cleanup function
cleanup() {
    rm -f "$TEMP_FILE" "$RESPONSE_FILE"
}
trap cleanup EXIT INT TERM ERR

# Parse arguments
if [ $# -lt 2 ]; then
    log_error "Usage: $0 <file_path> <content>"
    exit 1
fi

FILE_PATH="$1"
CONTENT="$2"

# Check if MCP server is running (health endpoint is always public)
if ! curl -s -f "$SYNAPSE_MCP_URL/health" > /dev/null 2>&1; then
    log_warning "Synapse MCP server not running at $SYNAPSE_MCP_URL"
    log_info "Starting in standalone mode (no rule validation)"
    echo "$CONTENT"  # Pass through content unchanged
    exit 0
fi

log_info "Validating content for: $FILE_PATH"

# Prepare curl headers
CURL_HEADERS=("-H" "Content-Type: application/json")

# Add authentication header if token is available
if [ -n "$SYNAPSE_AUTH_TOKEN" ]; then
    CURL_HEADERS+=("-H" "Authorization: Bearer $SYNAPSE_AUTH_TOKEN")
    log_info "Using authentication token"
fi

# Send request to pre-write endpoint with secure JSON construction
if jq -n \
    --arg path "$FILE_PATH" \
    --arg content "$CONTENT" \
    '{"data": {"file_path": $path, "content": $content}}' | \
  curl -s --connect-timeout 5 --max-time 10 -X POST "$SYNAPSE_MCP_URL/enforce/pre-write" \
    "${CURL_HEADERS[@]}" \
    --data-binary @- \
    -o "$RESPONSE_FILE" 2>/dev/null; then
    
    # Parse response
    SUCCESS=$(jq -r '.success' "$RESPONSE_FILE" 2>/dev/null || echo "false")
    VALID=$(jq -r '.data.valid' "$RESPONSE_FILE" 2>/dev/null || echo "false")
    
    if [ "$SUCCESS" = "true" ] && [ "$VALID" = "true" ]; then
        log_success "Content passes all rule validations"
        echo "$CONTENT"  # Output original content
        exit 0
    elif [ "$SUCCESS" = "true" ] && [ "$VALID" = "false" ]; then
        # Get violations and auto-fixes
        VIOLATIONS=$(jq -r '.data.violations | length' "$RESPONSE_FILE" 2>/dev/null || echo "0")
        FIXED_CONTENT=$(jq -r '.data.fixed_content' "$RESPONSE_FILE" 2>/dev/null || echo "null")
        
        log_error "Content violates $VIOLATIONS rule(s)"
        
        # Show violations
        jq -r '.data.violations[] | "  - \(.rule_name): \(.message)"' "$RESPONSE_FILE" 2>/dev/null || true
        
        # Check if auto-fixes are available
        if [ "$FIXED_CONTENT" != "null" ] && [ "$FIXED_CONTENT" != "" ]; then
            log_info "Auto-fixes applied:"
            jq -r '.data.auto_fixes[]? | "  - \(.description) (confidence: \(.confidence))"' "$RESPONSE_FILE" 2>/dev/null || true
            echo "$FIXED_CONTENT"  # Output fixed content
            exit 0
        else
            log_info "Manual fixes required - see violations above"
            echo "$CONTENT"  # Output original content for manual fixing
            exit 1
        fi
    else
        log_error "Server error during validation"
        jq -r '.error // "Unknown error"' "$RESPONSE_FILE" >&2 2>/dev/null || log_error "Failed to parse error response"
        echo "$CONTENT"  # Pass through on server error
        exit 1
    fi
else
    log_error "Failed to connect to Synapse MCP server"
    log_info "Falling back to unvalidated mode"
    echo "$CONTENT"  # Pass through on connection failure
    exit 0
fi