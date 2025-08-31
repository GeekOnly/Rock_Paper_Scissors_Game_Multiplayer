# 🐳 Docker Load Testing Guide

This guide shows how to use Docker to run comprehensive load tests against the RPS Game Server.

## 🚀 Quick Start

### 1. Build Images
```bash
./test-docker.sh build
```

### 2. Start Server
```bash
./test-docker.sh start
```

### 3. Run Quick Test
```bash
./test-docker.sh quick --connections 5000
```

### 4. Find Server Limits
```bash
./test-docker.sh stress --max-conn 50000
```

## 📋 Available Commands

| Command | Description | Example |
|---------|-------------|---------|
| `build` | Build Docker images | `./test-docker.sh build` |
| `start` | Start RPS server only | `./test-docker.sh start` |
| `test` | Run full test suite | `./test-docker.sh test` |
| `quick` | Quick test with N connections | `./test-docker.sh quick --connections 10000` |
| `stress` | Stress test to find limits | `./test-docker.sh stress --max-conn 100000` |
| `stop` | Stop all services | `./test-docker.sh stop` |
| `clean` | Clean up everything | `./test-docker.sh clean` |
| `logs` | Show server logs | `./test-docker.sh logs` |
| `monitor` | Start with monitoring | `./test-docker.sh monitor` |

## 🧪 Test Types

### 1. Quick Test
Fast test with specified number of connections:
```bash
./test-docker.sh quick --connections 5000
```

### 2. Stress Test
Progressive load increase to find breaking point:
```bash
./test-docker.sh stress --max-conn 50000
```

### 3. Full Test Suite
Comprehensive testing including:
- Connection limits test
- 5000 concurrent connections
- Progressive load test
- 10-minute sustained load
- Stress test to breaking point

```bash
./test-docker.sh test
```

## 📊 Monitoring

Start with Prometheus and Grafana monitoring:
```bash
./test-docker.sh monitor
```

Access dashboards:
- **RPS Server**: http://localhost:8080
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

## 📁 Test Results

All test results are saved to `./test-results/` directory:
- Individual test logs
- Summary reports
- Performance metrics
- Error analysis

## 🔧 Advanced Usage

### Manual Docker Commands

Start server only:
```bash
docker-compose up -d rps-server
```

Run specific test:
```bash
docker-compose --profile testing run --rm load-tester /app/scripts/quick_test.sh
```

Check server health:
```bash
curl http://localhost:8080/health
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_URL` | `ws://rps-server:8080` | WebSocket server URL |
| `CONNECTIONS` | `1000` | Number of connections for quick test |
| `MAX_CONNECTIONS` | `20000` | Maximum connections for stress test |
| `RUST_LOG` | `info` | Logging level |

### Custom Test Configuration

Create custom test by running container interactively:
```bash
docker-compose --profile testing run --rm -it load-tester bash
```

Then run tests manually:
```bash
load_test --server ws://rps-server:8080 --connections 10000 --test-type concurrent
extreme_load_test --server ws://rps-server:8080 --test-type progressive
```

## 🐛 Troubleshooting

### Server Won't Start
```bash
./test-docker.sh logs
```

### Test Fails to Connect
Check if server is running:
```bash
docker-compose ps
curl http://localhost:8080/health
```

### Clean Everything and Restart
```bash
./test-docker.sh clean
./test-docker.sh build
./test-docker.sh start
```

### Resource Limits
If hitting system limits, adjust Docker resources:
- Increase memory limit in Docker Desktop
- Increase file descriptor limits
- Check system ulimits

## 📈 Performance Tips

1. **Use SSD storage** for better I/O performance
2. **Increase Docker memory** allocation (4GB+ recommended)
3. **Close unnecessary applications** during testing
4. **Use dedicated test machine** for accurate results
5. **Monitor system resources** during tests

## 🎯 Expected Results

Based on previous tests, the server should handle:
- ✅ 5,000+ concurrent connections
- ✅ 100% success rate up to tested limits
- ✅ Sub-30ms response times
- ✅ Zero connection drops
- ✅ Sustained load for extended periods

The actual limits will depend on your system resources and configuration.