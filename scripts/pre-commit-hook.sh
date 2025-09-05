#!/bin/bash

# Synapse MCP Pre-commit Hook Template
# This script enforces rules before commits using the PatternEnforcer

set -e

# Configuration
SYNAPSE_BINARY="${SYNAPSE_BINARY:-synapse}"
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

# Get list of changed files
CHANGED_FILES=$(git diff --cached --name-only)

if [ -z "$CHANGED_FILES" ]; then
    log_info "No files to check"
    exit 0
fi

log_info "Synapse MCP Rule Enforcement"
log_info "Checking $(echo "$CHANGED_FILES" | wc -l) changed files..."

if [ "$SYNAPSE_VERBOSE" = "true" ]; then
    echo "Changed files:"
    for file in $CHANGED_FILES; do
        echo "  • $file"
    done
fi

# Check if synapse binary is available
if ! command -v $SYNAPSE_BINARY &> /dev/null; then
    log_error "Synapse binary '$SYNAPSE_BINARY' not found"
    log_info "Please ensure Synapse MCP is built and in your PATH"
    log_info "Run: cargo build --release && export PATH=\"\$PWD/target/release:\$PATH\""
    exit 1
fi

# Run rule enforcement check
if [ "$SYNAPSE_VERBOSE" = "true" ]; then
    CHECK_CMD="$SYNAPSE_BINARY check --verbose --files $CHANGED_FILES"
else
    CHECK_CMD="$SYNAPSE_BINARY check --files $CHANGED_FILES"
fi

if [ "$SYNAPSE_VERBOSE" = "true" ]; then
    log_info "Running: $CHECK_CMD"
fi

if eval $CHECK_CMD; then
    log_success "All files pass rule enforcement"
    exit 0
else
    log_error "Rule enforcement failed"
    echo
    log_info "💡 To bypass this check (not recommended):"
    log_info "    git commit --no-verify"
    echo
    log_info "📋 To see detailed violations:"
    log_info "    $SYNAPSE_BINARY check --verbose --files $CHANGED_FILES"
    echo
    log_info "🔧 To test rules without enforcement:"
    log_info "    $SYNAPSE_BINARY check --dry-run --files $CHANGED_FILES"
    exit 1
fi