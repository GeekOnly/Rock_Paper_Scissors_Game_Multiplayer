#!/bin/bash

# Stress Test Script - Find Server Limits
# This script progressively increases load until failure

set -e

SERVER_URL=${SERVER_URL:-"ws://rps-server:8080"}
START_CONNECTIONS=${START_CONNECTIONS:-1000}
MAX_CONNECTIONS=${MAX_CONNECTIONS:-20000}
STEP=${STEP:-1000}

echo "💀 STRESS TEST - Finding Server Limits"
echo "======================================"
echo "Server: $SERVER_URL"
echo "Start: $START_CONNECTIONS connections"
echo "Max: $MAX_CONNECTIONS connections"
echo "Step: $STEP connections"
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

CURRENT=$START_CONNECTIONS
LAST_SUCCESS=0

while [ $CURRENT -le $MAX_CONNECTIONS ]; do
    echo ""
    echo "🔥 Testing $CURRENT concurrent connections..."
    
    if load_test --server "$SERVER_URL" --connections "$CURRENT" --test-type concurrent --duration 30; then
        echo "✅ $CURRENT connections: SUCCESS"
        LAST_SUCCESS=$CURRENT
    else
        echo "❌ $CURRENT connections: FAILED"
        echo ""
        echo "💥 LIMIT FOUND!"
        echo "🏆 Maximum successful connections: $LAST_SUCCESS"
        echo "💔 First failure at: $CURRENT connections"
        break
    fi
    
    CURRENT=$((CURRENT + STEP))
    
    # Cool down between tests
    echo "⏱️  Cooling down for 5 seconds..."
    sleep 5
done

if [ $CURRENT -gt $MAX_CONNECTIONS ]; then
    echo ""
    echo "🤯 NO LIMIT FOUND!"
    echo "🏆 Server handled up to $LAST_SUCCESS connections successfully"
    echo "🚀 Consider testing with higher limits!"
fi