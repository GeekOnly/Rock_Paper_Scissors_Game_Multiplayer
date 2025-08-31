#!/bin/bash

# Quick Load Test Script
# For rapid testing during development

set -e

SERVER_URL=${SERVER_URL:-"ws://rps-server:8080"}
CONNECTIONS=${CONNECTIONS:-1000}

echo "🚀 Quick Load Test"
echo "=================="
echo "Server: $SERVER_URL"
echo "Connections: $CONNECTIONS"
echo ""

# Wait for server
echo "⏳ Waiting for server..."
for i in {1..15}; do
    if curl -f http://rps-server:8080/health >/dev/null 2>&1; then
        echo "✅ Server ready!"
        break
    fi
    sleep 1
done

# Run quick test
echo "🔥 Running quick test with $CONNECTIONS connections..."
load_test --server "$SERVER_URL" --connections "$CONNECTIONS" --test-type concurrent

echo "✅ Quick test completed!"