#!/bin/bash

# Docker Load Testing Script
# Easy way to run load tests using Docker

set -e

echo "🐳 RPS Game Server Docker Load Testing"
echo "======================================"

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  build       Build Docker images"
    echo "  start       Start RPS server"
    echo "  test        Run full test suite"
    echo "  quick       Run quick test (1000 connections)"
    echo "  stress      Run stress test to find limits"
    echo "  stop        Stop all services"
    echo "  clean       Clean up containers and images"
    echo "  logs        Show server logs"
    echo "  monitor     Start with monitoring (Prometheus + Grafana)"
    echo "  network     Start server for network access (use test-network.sh for clients)"
    echo ""
    echo "Options:"
    echo "  --connections N    Number of connections for quick test (default: 1000)"
    echo "  --max-conn N       Maximum connections for stress test (default: 20000)"
    echo ""
    echo "Examples:"
    echo "  $0 build                    # Build images"
    echo "  $0 start                    # Start server"
    echo "  $0 quick --connections 5000 # Quick test with 5000 connections"
    echo "  $0 stress --max-conn 50000  # Stress test up to 50000 connections"
    echo "  $0 monitor                  # Start with monitoring dashboard"
}

# Parse arguments
COMMAND=${1:-help}
CONNECTIONS=1000
MAX_CONNECTIONS=20000

shift || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --connections)
            CONNECTIONS="$2"
            shift 2
            ;;
        --max-conn)
            MAX_CONNECTIONS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

case $COMMAND in
    build)
        echo "🔨 Building Docker images..."
        docker-compose build
        echo "✅ Build completed!"
        ;;
        
    start)
        echo "🚀 Starting RPS server..."
        docker-compose up -d rps-server
        echo "⏳ Waiting for server to be ready..."
        sleep 5
        
        # Check if server is running
        if docker-compose ps rps-server | grep -q "Up"; then
            echo "✅ RPS server is running on http://localhost:8080"
        else
            echo "❌ Failed to start RPS server"
            docker-compose logs rps-server
            exit 1
        fi
        ;;
        
    test)
        echo "🧪 Running full test suite..."
        docker-compose up -d rps-server
        sleep 5
        docker-compose --profile testing run --rm load-tester
        ;;
        
    quick)
        echo "⚡ Running quick test with $CONNECTIONS connections..."
        docker-compose up -d rps-server
        sleep 5
        docker-compose --profile testing run --rm -e CONNECTIONS=$CONNECTIONS load-tester /app/scripts/quick_test.sh
        ;;
        
    stress)
        echo "💀 Running stress test (max: $MAX_CONNECTIONS connections)..."
        docker-compose up -d rps-server
        sleep 5
        docker-compose --profile testing run --rm -e MAX_CONNECTIONS=$MAX_CONNECTIONS load-tester /app/scripts/stress_test.sh
        ;;
        
    stop)
        echo "🛑 Stopping all services..."
        docker-compose down
        echo "✅ All services stopped"
        ;;
        
    clean)
        echo "🧹 Cleaning up..."
        docker-compose down -v --rmi all
        docker system prune -f
        echo "✅ Cleanup completed"
        ;;
        
    logs)
        echo "📋 Showing server logs..."
        docker-compose logs -f rps-server
        ;;
        
    monitor)
        echo "📊 Starting with monitoring..."
        docker-compose --profile monitoring up -d
        echo "✅ Services started with monitoring:"
        echo "   🎮 RPS Server: http://localhost:8080"
        echo "   📈 Prometheus: http://localhost:9090"
        echo "   📊 Grafana: http://localhost:3000 (admin/admin)"
        ;;
        
    network)
        echo "🌐 Network testing mode - see test-network.sh for full options"
        echo "Quick network setup:"
        LOCAL_IP=$(hostname -I | awk '{print $1}' 2>/dev/null || echo "localhost")
        echo "   Server IP: $LOCAL_IP"
        echo "   Starting server for network access..."
        docker-compose -f docker-compose.network.yml up -d rps-server
        echo "✅ Server accessible at: http://$LOCAL_IP:8080"
        echo "💡 Use: ./test-network.sh for full network testing options"
        ;;
        
    help|*)
        show_usage
        ;;
esac