#!/bin/bash

# RPS Game Server Load Testing Suite
# This script runs comprehensive load tests against the RPS server

set -e

echo "🚀 Starting RPS Game Server Load Testing Suite"
echo "=============================================="

# Configuration
SERVER_URL=${SERVER_URL:-"ws://rps-server:8080"}
RESULTS_DIR="/app/results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# Create results directory
mkdir -p "$RESULTS_DIR"

# Wait for server to be ready
echo "⏳ Waiting for server to be ready..."
for i in {1..30}; do
    if curl -f http://rps-server:8080/health >/dev/null 2>&1; then
        echo "✅ Server is ready!"
        break
    fi
    echo "   Attempt $i/30 - Server not ready yet..."
    sleep 2
done

if ! curl -f http://rps-server:8080/health >/dev/null 2>&1; then
    echo "❌ Server failed to start within timeout"
    exit 1
fi

echo ""
echo "🧪 Running Test Suite"
echo "===================="

# Test 1: Connection Limits Test
echo "📊 Test 1: Connection Limits"
load_test --server "$SERVER_URL" --test-type limits 2>&1 | tee "$RESULTS_DIR/limits_test_$TIMESTAMP.log"

echo ""
echo "⏱️  Waiting 10 seconds between tests..."
sleep 10

# Test 2: Concurrent Connections Test (5000)
echo "🔥 Test 2: 5000 Concurrent Connections"
load_test --server "$SERVER_URL" --connections 5000 --test-type concurrent 2>&1 | tee "$RESULTS_DIR/concurrent_5k_$TIMESTAMP.log"

echo ""
echo "⏱️  Waiting 10 seconds between tests..."
sleep 10

# Test 3: Extreme Progressive Test
echo "💥 Test 3: Extreme Progressive Load Test"
extreme_load_test --server "$SERVER_URL" --test-type progressive --duration 120 2>&1 | tee "$RESULTS_DIR/extreme_progressive_$TIMESTAMP.log"

echo ""
echo "⏱️  Waiting 10 seconds between tests..."
sleep 10

# Test 4: Sustained Load Test
echo "⚡ Test 4: Sustained Load (10 minutes)"
load_test --server "$SERVER_URL" --connections 1000 --duration 600 --test-type sustained 2>&1 | tee "$RESULTS_DIR/sustained_10min_$TIMESTAMP.log"

echo ""
echo "⏱️  Waiting 10 seconds between tests..."
sleep 10

# Test 5: Stress Test - Find Breaking Point
echo "💀 Test 5: Stress Test - Finding Breaking Point"
extreme_load_test --server "$SERVER_URL" --test-type stress --max-connections 10000 2>&1 | tee "$RESULTS_DIR/stress_test_$TIMESTAMP.log"

echo ""
echo "📈 Generating Test Summary"
echo "========================="

# Generate summary report
cat > "$RESULTS_DIR/test_summary_$TIMESTAMP.md" << EOF
# RPS Game Server Load Test Results

**Test Date:** $(date)
**Server URL:** $SERVER_URL
**Test Duration:** $(date)

## Test Results Summary

### 1. Connection Limits Test
- **File:** limits_test_$TIMESTAMP.log
- **Description:** Tests server capacity at various connection levels

### 2. 5000 Concurrent Connections Test  
- **File:** concurrent_5k_$TIMESTAMP.log
- **Description:** Tests 5000 simultaneous connections

### 3. Extreme Progressive Load Test
- **File:** extreme_progressive_$TIMESTAMP.log
- **Description:** Progressive load increase over 2 minutes

### 4. Sustained Load Test (10 minutes)
- **File:** sustained_10min_$TIMESTAMP.log
- **Description:** 1000 connections sustained for 10 minutes

### 5. Stress Test - Breaking Point
- **File:** stress_test_$TIMESTAMP.log
- **Description:** Finds maximum server capacity

## System Information
- **Container:** Load Tester
- **Network:** Docker Bridge
- **Test Suite Version:** 1.0

EOF

echo "✅ All tests completed!"
echo "📁 Results saved to: $RESULTS_DIR"
echo "📊 Summary report: test_summary_$TIMESTAMP.md"

# Keep container running for result inspection
echo ""
echo "🔍 Test container will remain running for result inspection..."
echo "   Use 'docker exec -it <container> bash' to inspect results"
echo "   Results are also mounted to ./test-results/ on host"

# Sleep to keep container alive
sleep 3600