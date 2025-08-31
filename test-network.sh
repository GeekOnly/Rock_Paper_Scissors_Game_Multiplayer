#!/bin/bash

# Network Load Testing Script
# For testing across multiple machines on the same WiFi network

set -e

echo "🌐 RPS Game Server Network Load Testing"
echo "======================================="

# Function to get local IP
get_local_ip() {
    # Try different methods to get local IP
    if command -v hostname >/dev/null 2>&1; then
        hostname -I | awk '{print $1}' 2>/dev/null || \
        ip route get 1.1.1.1 | awk '{print $7}' 2>/dev/null || \
        ifconfig | grep -Eo 'inet (addr:)?([0-9]*\.){3}[0-9]*' | grep -Eo '([0-9]*\.){3}[0-9]*' | grep -v '127.0.0.1' | head -1
    else
        echo "192.168.1.100"  # Fallback
    fi
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  server          Start server for network access"
    echo "  client          Run as network client (specify server IP)"
    echo "  distributed     Run distributed test with multiple clients"
    echo "  info            Show network information"
    echo "  firewall        Show firewall configuration help"
    echo ""
    echo "Options:"
    echo "  --server-ip IP     Server IP address (for client mode)"
    echo "  --connections N    Number of connections per client (default: 1000)"
    echo "  --clients N        Number of distributed clients (default: 3)"
    echo ""
    echo "Examples:"
    echo "  # On server machine:"
    echo "  $0 server"
    echo ""
    echo "  # On client machine:"
    echo "  $0 client --server-ip 192.168.1.100 --connections 2000"
    echo ""
    echo "  # Distributed test (multiple clients):"
    echo "  $0 distributed --server-ip 192.168.1.100 --clients 5 --connections 1000"
}

# Parse arguments
COMMAND=${1:-help}
SERVER_IP=""
CONNECTIONS=1000
CLIENTS=3

shift || true
while [[ $# -gt 0 ]]; do
    case $1 in
        --server-ip)
            SERVER_IP="$2"
            shift 2
            ;;
        --connections)
            CONNECTIONS="$2"
            shift 2
            ;;
        --clients)
            CLIENTS="$2"
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
    server)
        LOCAL_IP=$(get_local_ip)
        echo "🚀 Starting RPS server for network access..."
        echo "📍 Server will be accessible at:"
        echo "   Local: http://localhost:8080"
        echo "   Network: http://$LOCAL_IP:8080"
        echo "   WebSocket: ws://$LOCAL_IP:8080"
        echo ""
        echo "🔥 Share this IP with other machines: $LOCAL_IP"
        echo ""
        
        # Start server with network configuration
        docker-compose -f docker-compose.network.yml up -d rps-server
        
        echo "⏳ Waiting for server to start..."
        sleep 5
        
        # Test local access
        if curl -f http://localhost:8080/health >/dev/null 2>&1; then
            echo "✅ Server is running and accessible!"
            echo ""
            echo "📋 Server Status:"
            echo "   Health: http://localhost:8080/health"
            echo "   Logs: docker-compose -f docker-compose.network.yml logs -f rps-server"
            echo ""
            echo "🌐 Network Access Instructions:"
            echo "   1. Share this IP with other machines: $LOCAL_IP"
            echo "   2. On client machines, run:"
            echo "      $0 client --server-ip $LOCAL_IP --connections 1000"
            echo ""
            echo "🔒 If clients can't connect, check firewall settings:"
            echo "   $0 firewall"
        else
            echo "❌ Server failed to start"
            docker-compose -f docker-compose.network.yml logs rps-server
            exit 1
        fi
        ;;
        
    client)
        if [ -z "$SERVER_IP" ]; then
            echo "❌ Server IP is required for client mode"
            echo "Usage: $0 client --server-ip <SERVER_IP>"
            exit 1
        fi
        
        echo "🔌 Connecting to server at $SERVER_IP:8080"
        echo "🧪 Testing with $CONNECTIONS connections"
        echo ""
        
        # Test connectivity first
        echo "🔍 Testing server connectivity..."
        if curl -f "http://$SERVER_IP:8080/health" >/dev/null 2>&1; then
            echo "✅ Server is reachable!"
        else
            echo "❌ Cannot reach server at $SERVER_IP:8080"
            echo ""
            echo "💡 Troubleshooting:"
            echo "   1. Verify server IP: $SERVER_IP"
            echo "   2. Check if server is running on target machine"
            echo "   3. Check firewall settings on server machine"
            echo "   4. Try ping: ping $SERVER_IP"
            exit 1
        fi
        
        # Run network load test
        echo "🚀 Starting network load test..."
        export SERVER_IP=$SERVER_IP
        export CONNECTIONS=$CONNECTIONS
        docker-compose -f docker-compose.network.yml --profile network-testing run --rm network-tester
        ;;
        
    distributed)
        if [ -z "$SERVER_IP" ]; then
            echo "❌ Server IP is required for distributed mode"
            echo "Usage: $0 distributed --server-ip <SERVER_IP> --clients <N>"
            exit 1
        fi
        
        echo "🌐 Distributed Load Test"
        echo "======================="
        echo "Server: $SERVER_IP:8080"
        echo "Clients: $CLIENTS"
        echo "Connections per client: $CONNECTIONS"
        echo "Total connections: $((CLIENTS * CONNECTIONS))"
        echo ""
        
        # Test connectivity
        if ! curl -f "http://$SERVER_IP:8080/health" >/dev/null 2>&1; then
            echo "❌ Cannot reach server at $SERVER_IP:8080"
            exit 1
        fi
        
        echo "🚀 Starting $CLIENTS distributed clients..."
        
        # Start multiple client instances
        for i in $(seq 1 $CLIENTS); do
            echo "   Starting client $i/$CLIENTS..."
            export SERVER_IP=$SERVER_IP
            export CONNECTIONS=$CONNECTIONS
            export TESTER_ID=$i
            docker-compose -f docker-compose.network.yml --profile distributed run -d distributed-tester &
        done
        
        echo "⏳ All clients started, waiting for completion..."
        wait
        echo "✅ Distributed test completed!"
        ;;
        
    info)
        LOCAL_IP=$(get_local_ip)
        echo "🌐 Network Information"
        echo "====================="
        echo "Local IP: $LOCAL_IP"
        echo "Available interfaces:"
        ip addr show | grep -E "inet.*scope global" || ifconfig | grep -E "inet.*broadcast" || echo "Could not determine network interfaces"
        echo ""
        echo "🔍 To test from another machine:"
        echo "   1. Start server: $0 server"
        echo "   2. On client: $0 client --server-ip $LOCAL_IP"
        ;;
        
    firewall)
        echo "🔒 Firewall Configuration Help"
        echo "=============================="
        echo ""
        echo "🐧 Linux (Ubuntu/Debian):"
        echo "   sudo ufw allow 8080"
        echo "   sudo ufw status"
        echo ""
        echo "🍎 macOS:"
        echo "   # Usually no action needed for local network"
        echo "   # Check System Preferences > Security & Privacy > Firewall"
        echo ""
        echo "🪟 Windows:"
        echo "   # Windows Defender Firewall"
        echo "   netsh advfirewall firewall add rule name=\"RPS Server\" dir=in action=allow protocol=TCP localport=8080"
        echo ""
        echo "🐳 Docker Desktop:"
        echo "   # Usually handles port forwarding automatically"
        echo "   # Make sure Docker Desktop is running"
        echo ""
        echo "🔍 Testing connectivity:"
        echo "   # From client machine:"
        echo "   telnet <SERVER_IP> 8080"
        echo "   # or"
        echo "   curl http://<SERVER_IP>:8080/health"
        ;;
        
    help|*)
        show_usage
        ;;
esac