---
mcp: synapse
type: rule
title: "Security Rules"
tags: ["security", "compliance"]
---

# Security Rules

FORBIDDEN: `password` - Never hardcode passwords in source code
FORBIDDEN: `secret_key` - Never hardcode secret keys in source code
FORBIDDEN: `/http://[^s]/` - Use HTTPS for all external communications
REQUIRED: `validate_input` - All user inputs must be validated
STANDARD: `sanitize` - Sanitize all user data before processing
CONVENTION: `auth_` - Authentication functions should be prefixed with auth_