#!/bin/bash

# Network Load Testing Script
# Test server from multiple machines on the same network

set -e

# Get server IP (can be overridden)
SERVER_IP=${SERVER_IP:-$(hostname -I | awk '{print $1}')}
SERVER_PORT=${SERVER_PORT:-8080}
SERVER_URL="ws://${SERVER_IP}:${SERVER_PORT}"

echo "🌐 Network Load Testing"
echo "======================"
echo "Server IP: $SERVER_IP"
echo "Server Port: $SERVER_PORT"
echo "WebSocket URL: $SERVER_URL"
echo ""

# Test server connectivity first
echo "🔍 Testing server connectivity..."
if curl -f "http://${SERVER_IP}:${SERVER_PORT}/health" >/dev/null 2>&1; then
    echo "✅ Server is accessible from network!"
else
    echo "❌ Cannot reach server at http://${SERVER_IP}:${SERVER_PORT}"
    echo ""
    echo "💡 Troubleshooting tips:"
    echo "   1. Make sure server is running: ./test-docker.sh start"
    echo "   2. Check firewall settings"
    echo "   3. Verify IP address: $SERVER_IP"
    echo "   4. Try from server machine: curl http://localhost:8080/health"
    exit 1
fi

echo ""
echo "🧪 Running network load test..."

# Run load test with network URL
CONNECTIONS=${CONNECTIONS:-1000}
load_test --server "$SERVER_URL" --connections "$CONNECTIONS" --test-type concurrent

echo ""
echo "✅ Network test completed!"
echo "📊 Server handled $CONNECTIONS connections from network client"