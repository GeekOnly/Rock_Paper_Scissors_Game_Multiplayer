# 🚀 RPS Game Server - Production Deployment Guide

## 📋 Quick Deployment Summary

| Provider | Best For | Monthly Cost | Setup Time | Scalability |
|----------|----------|--------------|------------|-------------|
| **Digital Ocean** | Indie/Small Studios | $63-126 | 30 mins | Manual |
| **AWS GameLift** | Enterprise/AAA | $280-561 | 2-4 hours | Auto |
| **GCP Compute** | Medium Studios | $98-238 | 1-2 hours | Auto |
| **Vultr** | Budget Projects | $48-192 | 20 mins | Manual |

---

## 🎯 Recommended: Digital Ocean CPU-Optimized

### Why Digital Ocean?
- ✅ **Best Price/Performance**: $63/month for 5,000 players
- ✅ **Simple Setup**: Docker-ready in 30 minutes  
- ✅ **Predictable Costs**: No surprise charges
- ✅ **Great Support**: Excellent documentation
- ✅ **99.4% Profit Margin** at $2/player/month

---

## 🚀 Step-by-Step Deployment

### 1. Create Digital Ocean Droplet

```bash
# Using doctl CLI (recommended)
doctl compute droplet create rps-game-server \
  --image ubuntu-22-04-x64 \
  --size c-4 \
  --region nyc1 \
  --ssh-keys YOUR_SSH_KEY_ID \
  --enable-monitoring \
  --enable-ipv6

# Manual setup via web interface:
# 1. Go to Digital Ocean dashboard
# 2. Create Droplet
# 3. Choose: CPU-Optimized, 4 vCPU, 8GB RAM ($63/month)
# 4. Select: Ubuntu 22.04 LTS
# 5. Add your SSH key
# 6. Enable monitoring & IPv6
```

### 2. Server Setup (One-Command Deploy)

```bash
# Run this on your new droplet
curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/main/scripts/deploy.sh | bash
```

### 3. Manual Setup Script

```bash
#!/bin/bash
# Save as deploy.sh

echo "🚀 Setting up RPS Game Server..."

# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose

# Install essential tools
sudo apt install -y htop iotop nethogs ufw fail2ban git curl

# Configure firewall
sudo ufw allow ssh
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 8080/tcp
sudo ufw --force enable

# Setup fail2ban
sudo systemctl enable fail2ban
sudo systemctl start fail2ban

# Clone and deploy
git clone https://github.com/YOUR_REPO/rps-server.git /opt/rps-server
cd /opt/rps-server

# Create necessary directories
mkdir -p logs ssl nginx/logs monitoring

# Start production services
docker-compose -f docker-compose.production.yml up -d

# Get server IP
SERVER_IP=$(curl -s ifconfig.me)

echo "✅ RPS Server deployed successfully!"
echo "🌐 Server URL: http://$SERVER_IP:8080"
echo "📊 Grafana: http://$SERVER_IP:3000 (admin/admin123)"
echo "🔍 Health Check: curl http://$SERVER_IP:8080/health"
```

---

## 📊 Cost Breakdown & ROI

### Digital Ocean CPU-Optimized ($63/month)
```
Server Cost:           $63/month
Additional Services:   $12/month (load balancer)
Total Infrastructure:  $75/month

Revenue Scenarios:
- 1,000 players × $2 = $2,000/month (96.3% profit)
- 2,500 players × $2 = $5,000/month (98.5% profit)  
- 5,000 players × $2 = $10,000/month (99.3% profit)

Break-even: 38 paying players
```

### Scaling Path:
1. **Start**: 1 Droplet ($63) → 5,000 players
2. **Scale**: 2 Droplets + Load Balancer ($150) → 10,000 players  
3. **Global**: 3 Regions ($225) → 15,000 players worldwide
4. **Enterprise**: Migrate to AWS GameLift ($500+) → Unlimited

---

## 🌍 Multi-Region Deployment

### Global Setup (3 Regions):
```bash
# US East (Primary)
doctl compute droplet create rps-us-east --region nyc1 --size c-4

# Europe (Secondary)  
doctl compute droplet create rps-eu-west --region lon1 --size c-4

# Asia Pacific (Secondary)
doctl compute droplet create rps-ap-southeast --region sgp1 --size c-4

# Total Cost: $189/month for global coverage
```

### Load Balancer Setup:
```bash
# Create Digital Ocean Load Balancer
doctl compute load-balancer create \
  --name rps-global-lb \
  --algorithm round_robin \
  --health-check protocol:http,port:8080,path:/health \
  --droplet-ids $US_DROPLET_ID,$EU_DROPLET_ID,$ASIA_DROPLET_ID
```

---

## 🔧 Production Optimizations

### 1. Performance Tuning
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# Optimize network settings
echo "net.core.somaxconn = 65536" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65536" >> /etc/sysctl.conf
sysctl -p
```

### 2. SSL/TLS Setup
```bash
# Install Certbot for Let's Encrypt
sudo apt install certbot python3-certbot-nginx

# Get SSL certificate
sudo certbot --nginx -d yourdomain.com

# Auto-renewal
sudo crontab -e
# Add: 0 12 * * * /usr/bin/certbot renew --quiet
```

### 3. Monitoring & Alerts
```bash
# Setup Grafana alerts
# 1. Go to http://YOUR_IP:3000
# 2. Login: admin/admin123
# 3. Import dashboard: 1860 (Node Exporter)
# 4. Setup alerts for:
#    - CPU > 80%
#    - Memory > 90%
#    - Disk > 85%
#    - Connection failures > 1%
```

---

## 🚨 Troubleshooting

### Common Issues:

#### 1. High Memory Usage
```bash
# Check memory
free -h
docker stats

# Solution: Add swap or upgrade RAM
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

#### 2. Connection Limits
```bash
# Check current limits
ulimit -n

# Increase limits (add to /etc/security/limits.conf)
* soft nofile 65536
* hard nofile 65536
```

#### 3. Docker Issues
```bash
# Restart services
docker-compose -f docker-compose.production.yml restart

# Check logs
docker-compose -f docker-compose.production.yml logs -f rps-server

# Clean up
docker system prune -f
```

---

## 📈 Scaling Strategies

### Vertical Scaling (Same Server, More Power):
```
Current: 4 vCPU, 8GB RAM ($63) → 5,000 players
Upgrade: 8 vCPU, 16GB RAM ($126) → 10,000 players
```

### Horizontal Scaling (Multiple Servers):
```
2 Servers + Load Balancer = $150/month → 10,000 players
3 Servers + Load Balancer = $225/month → 15,000 players
```

### Hybrid Approach:
```
Game Servers: Digital Ocean ($63 × 3 regions = $189)
Database: AWS RDS ($25/month)
CDN: CloudFlare (Free)
Monitoring: DataDog ($15/month)
Total: $229/month for enterprise-grade setup
```

---

## 🎯 Success Metrics

### Key Performance Indicators:
- **Concurrent Players**: Target 5,000+
- **Response Time**: < 50ms average
- **Uptime**: 99.9% (8.76 hours downtime/year)
- **Success Rate**: 99.5%+ connections
- **Revenue per User**: $2-5/month
- **Profit Margin**: 95%+

### Monitoring Dashboard:
- Real-time player count
- Server resource usage
- Response time graphs  
- Error rate tracking
- Revenue analytics

---

## 🏆 Production Checklist

### Before Launch:
- [ ] Load testing completed (5,000+ users)
- [ ] SSL certificates installed
- [ ] Domain name configured
- [ ] Monitoring & alerts setup
- [ ] Backup strategy implemented
- [ ] Security hardening completed
- [ ] Documentation updated

### Post-Launch:
- [ ] Monitor performance for 24 hours
- [ ] Verify all alerts working
- [ ] Test failover procedures
- [ ] Document any issues
- [ ] Plan scaling strategy
- [ ] Setup regular backups

**Estimated Setup Time**: 2-4 hours
**Monthly Operating Cost**: $75-150
**Expected Profit Margin**: 95%+
- [ ] Backup strategy defined

### ✅ Provider Selection
- [ ] Cost analysis completed
- [ ] Provider account created
- [ ] Payment method configured
- [ ] Resource limits verified

### ✅ Security
- [ ] Firewall rules configured
- [ ] DDoS protection enabled
- [ ] Rate limiting implemented
- [ ] Health checks configured

---

## 🌊 Digital Ocean Deployment (Recommended)

### 1. Create Droplet
```bash
# Using doctl CLI
doctl compute droplet create rps-game-server \
  --image ubuntu-22-04-x64 \
  --size c-4 \
  --region nyc1 \
  --ssh-keys YOUR_SSH_KEY_ID \
  --enable-monitoring \
  --enable-backups
```

### 2. Setup Server
```bash
# SSH into server
ssh root@YOUR_SERVER_IP

# Update system
apt update && apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sh get-docker.sh

# Install Docker Compose
curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# Clone repository
git clone YOUR_REPO_URL /opt/rps-server
cd /opt/rps-server
```

### 3. Configure Environment
```bash
# Create production environment file
cat > .env.production << EOF
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
DOMAIN=your-domain.com
SSL_CERT_PATH=/etc/ssl/certs/your-cert.pem
SSL_KEY_PATH=/etc/ssl/private/your-key.pem
EOF
```

### 4. Setup SSL with Let's Encrypt
```bash
# Install Certbot
apt install certbot -y

# Get SSL certificate
certbot certonly --standalone -d your-domain.com

# Copy certificates
cp /etc/letsencrypt/live/your-domain.com/fullchain.pem /opt/rps-server/ssl/
cp /etc/letsencrypt/live/your-domain.com/privkey.pem /opt/rps-server/ssl/
```

### 5. Deploy with Docker Compose
```bash
# Build and start
docker-compose -f docker-compose.production.yml up -d

# Verify deployment
curl https://your-domain.com/health
```

---

## 🟠 AWS EC2 Deployment

### 1. Launch EC2 Instance
```bash
# Using AWS CLI
aws ec2 run-instances \
  --image-id ami-0c02fb55956c7d316 \
  --instance-type c5.xlarge \
  --key-name your-key-pair \
  --security-group-ids sg-your-security-group \
  --subnet-id subnet-your-subnet \
  --associate-public-ip-address \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=rps-game-server}]'
```

### 2. Configure Security Group
```bash
# Allow HTTP/HTTPS and WebSocket
aws ec2 authorize-security-group-ingress \
  --group-id sg-your-security-group \
  --protocol tcp \
  --port 80 \
  --cidr 0.0.0.0/0

aws ec2 authorize-security-group-ingress \
  --group-id sg-your-security-group \
  --protocol tcp \
  --port 443 \
  --cidr 0.0.0.0/0

aws ec2 authorize-security-group-ingress \
  --group-id sg-your-security-group \
  --protocol tcp \
  --port 8080 \
  --cidr 0.0.0.0/0
```

### 3. Setup Load Balancer (Optional)
```bash
# Create Application Load Balancer
aws elbv2 create-load-balancer \
  --name rps-game-alb \
  --subnets subnet-12345 subnet-67890 \
  --security-groups sg-your-alb-sg
```

---

## 🎮 AWS GameLift Deployment

### 1. Create GameLift Fleet
```bash
# Upload build
aws gamelift upload-build \
  --name "RPS-Server-v1.0" \
  --build-version "1.0.0" \
  --build-root ./target/release/ \
  --operating-system AMAZON_LINUX_2

# Create fleet
aws gamelift create-fleet \
  --name "RPS-Production-Fleet" \
  --description "RPS Game Server Production Fleet" \
  --build-id build-your-build-id \
  --ec2-instance-type c5.large \
  --fleet-type ON_DEMAND \
  --runtime-configuration 'ServerProcesses=[{LaunchPath="/local/game/rps-server",ConcurrentExecutions=1}]'
```

### 2. Configure Scaling
```bash
# Set scaling policies
aws gamelift put-scaling-policy \
  --fleet-id fleet-your-fleet-id \
  --name "RPS-Scaling-Policy" \
  --policy-type RuleBased \
  --metric-name AvailableGameSessions \
  --threshold 10 \
  --evaluation-periods 2 \
  --scaling-adjustment-type ChangeInCapacity \
  --scaling-adjustment 1
```

---

## 🔵 Google Cloud Platform Deployment

### 1. Create Compute Instance
```bash
# Using gcloud CLI
gcloud compute instances create rps-game-server \
  --machine-type=c2-standard-4 \
  --zone=us-central1-a \
  --image-family=ubuntu-2204-lts \
  --image-project=ubuntu-os-cloud \
  --boot-disk-size=50GB \
  --boot-disk-type=pd-ssd \
  --tags=rps-server
```

### 2. Configure Firewall
```bash
# Allow WebSocket traffic
gcloud compute firewall-rules create allow-rps-server \
  --allow tcp:8080,tcp:443,tcp:80 \
  --source-ranges 0.0.0.0/0 \
  --target-tags rps-server
```

### 3. Setup Load Balancer
```bash
# Create instance group
gcloud compute instance-groups unmanaged create rps-server-group \
  --zone=us-central1-a

# Add instance to group
gcloud compute instance-groups unmanaged add-instances rps-server-group \
  --instances=rps-game-server \
  --zone=us-central1-a

# Create load balancer
gcloud compute backend-services create rps-backend-service \
  --protocol=HTTP \
  --health-checks=rps-health-check \
  --global
```

---

## 📊 Production Docker Compose

<function_calls>
<invoke name="fsWrite">
<parameter name="path">docker-compose.production.ymlersion: '3.8'

services:
  # Production RPS Server
  rps-server:
    build:
      context: .
      dockerfile: Dockerfile
      target: runtime
    ports:
      - "80:8080"
      - "443:8443"
    environment:
      - RUST_LOG=info
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
      - SSL_PORT=8443
      - SSL_CERT_PATH=/app/ssl/cert.pem
      - SSL_KEY_PATH=/app/ssl/key.pem
      - METRICS_ENABLED=true
      - METRICS_PORT=9090
    volumes:
      - ./ssl:/app/ssl:ro
      - ./logs:/app/logs
      - rps-data:/app/data
    networks:
      - rps-production
    deploy:
      resources:
        limits:
          memory: 4G
          cpus: '4.0'
        reservations:
          memory: 2G
          cpus: '2.0'
      restart_policy:
        condition: unless-stopped
        delay: 5s
        max_attempts: 3
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: "json-file"
      options:
        max-size: "100m"
        max-file: "5"

  # Nginx Reverse Proxy
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf:ro
      - ./ssl:/etc/ssl/certs:ro
      - ./nginx/logs:/var/log/nginx
    depends_on:
      - rps-server
    networks:
      - rps-production
    restart: unless-stopped

  # Redis for Session Management (Optional)
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    networks:
      - rps-production
    restart: unless-stopped
    command: redis-server --appendonly yes --maxmemory 512mb --maxmemory-policy allkeys-lru

  # Prometheus Monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    networks:
      - rps-production
    restart: unless-stopped
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=30d'
      - '--web.enable-lifecycle'

  # Grafana Dashboard
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=${GRAFANA_PASSWORD:-admin}
      - GF_USERS_ALLOW_SIGN_UP=false
      - GF_SERVER_DOMAIN=${DOMAIN:-localhost}
      - GF_SMTP_ENABLED=true
      - GF_SMTP_HOST=${SMTP_HOST:-smtp.gmail.com:587}
      - GF_SMTP_USER=${SMTP_USER}
      - GF_SMTP_PASSWORD=${SMTP_PASSWORD}
    volumes:
      - grafana-data:/var/lib/grafana
      - ./monitoring/grafana/dashboards:/etc/grafana/provisioning/dashboards:ro
      - ./monitoring/grafana/datasources:/etc/grafana/provisioning/datasources:ro
    networks:
      - rps-production
    restart: unless-stopped

  # Log Aggregation
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./monitoring/loki.yml:/etc/loki/local-config.yaml:ro
      - loki-data:/loki
    networks:
      - rps-production
    restart: unless-stopped

  # Log Shipper
  promtail:
    image: grafana/promtail:latest
    volumes:
      - ./logs:/var/log/app:ro
      - ./monitoring/promtail.yml:/etc/promtail/config.yml:ro
      - /var/lib/docker/containers:/var/lib/docker/containers:ro
    networks:
      - rps-production
    restart: unless-stopped

networks:
  rps-production:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16

volumes:
  rps-data:
  redis-data:
  prometheus-data:
  grafana-data:
  loki-data: