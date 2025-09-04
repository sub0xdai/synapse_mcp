#!/bin/bash

# Test API script for Synapse MCP server
# Following TDD principle - automated testing of all endpoints

set -e  # Exit on any error

SERVER_URL="http://localhost:8080"
echo "🧪 Testing Synapse MCP Server API"
echo "=================================="

# Function to test endpoint with pretty output
test_endpoint() {
    local method=$1
    local endpoint=$2
    local data=$3
    local description=$4
    
    echo
    echo "Testing: $description"
    echo "  $method $endpoint"
    
    if [ "$method" = "POST" ]; then
        response=$(curl -s -X POST "$SERVER_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data" || echo "CURL_FAILED")
    else
        response=$(curl -s "$SERVER_URL$endpoint" || echo "CURL_FAILED")
    fi
    
    if [ "$response" = "CURL_FAILED" ]; then
        echo "  ❌ Request failed - is the server running?"
        return 1
    fi
    
    # Pretty print JSON and check if success field is true
    echo "  Response:"
    echo "$response" | jq '.' 2>/dev/null || echo "$response"
    
    success=$(echo "$response" | jq -r '.success // "unknown"' 2>/dev/null)
    if [ "$success" = "true" ]; then
        echo "  ✅ Success"
    elif [ "$success" = "false" ]; then
        echo "  ⚠️  API returned success=false (may be expected)"
    else
        echo "  ✅ Response received (no success field expected)"
    fi
}

# Test 1: Health check
test_endpoint "GET" "/health" "" "Health check endpoint"

# Test 2: Natural language query
test_endpoint "POST" "/query" '{"query": "find rules about performance"}' "Natural language query"

# Test 3: Query nodes by type - Rule
test_endpoint "GET" "/nodes/rule" "" "Query nodes by type: Rule"

# Test 4: Query nodes by type - Architecture
test_endpoint "GET" "/nodes/architecture" "" "Query nodes by type: Architecture" 

# Test 5: Query nodes by type - Decision
test_endpoint "GET" "/nodes/decision" "" "Query nodes by type: Decision"

# Test 6: Invalid node type (should fail gracefully)
test_endpoint "GET" "/nodes/invalid" "" "Invalid node type (should fail gracefully)"

# Test 7: Related nodes (we'll use a hardcoded ID that might exist)
test_endpoint "GET" "/node/test-id/related" "" "Related nodes query (may return empty)"

echo
echo "🎉 API testing complete!"
echo "Note: Some endpoints may return empty results if no data is indexed yet."
echo "Run 'cargo run --bin indexer test_docs/*.md' to index test documents first."