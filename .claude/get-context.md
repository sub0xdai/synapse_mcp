# Context Onboard Slash Command

This slash command retrieves all context files from the `.claude/context/` directory for onboarding.

## Command Definition

```yaml
name: context:onboard
description: Onboard by retrieving all context from .claude/context/ directory
```

## Implementation

```bash
#!/bin/bash

# Get Context Slash Command
# Retrieves all context files from .claude/context/ directory

CONTEXT_DIR=".claude/context"

if [ ! -d "$CONTEXT_DIR" ]; then
    echo "❌ Context directory not found: $CONTEXT_DIR"
    exit 1
fi

echo "📁 Retrieving all context from $CONTEXT_DIR"
echo "================================================="

# Find all markdown files in context directory
find "$CONTEXT_DIR" -name "*.md" -type f | sort | while read -r file; do
    if [ -f "$file" ]; then
        echo ""
        echo "📄 **$(basename "$file")**"
        echo "---"
        cat "$file"
        echo ""
        echo "================================================="
    fi
done

echo ""
echo "✅ Context retrieval complete"
```