#!/bin/bash

# AWS Graviton Deployment Script for RPS Game Server
# Optimized for ARM64 architecture and high performance

set -e

echo "🚀 AWS Graviton RPS Server Deployment"
echo "====================================="

# Configuration
REGION=${AWS_REGION:-us-east-1}
INSTANCE_TYPE=${INSTANCE_TYPE:-c7g.xlarge}
KEY_NAME=${KEY_NAME:-rps-server-key}
SECURITY_GROUP=${SECURITY_GROUP:-rps-server-sg}
IMAGE_NAME="rps-server:graviton"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check AWS CLI
    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI not found. Please install it first."
        exit 1
    fi
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        print_error "Docker not found. Please install it first."
        exit 1
    fi
    
    # Check AWS credentials
    if ! aws sts get-caller-identity &> /dev/null; then
        print_error "AWS credentials not configured. Run 'aws configure' first."
        exit 1
    fi
    
    print_success "Prerequisites check passed"
}

# Function to build multi-arch Docker image
build_image() {
    print_status "Building multi-architecture Docker image..."
    
    # Setup buildx if not exists
    if ! docker buildx ls | grep -q multiarch; then
        docker buildx create --name multiarch --use
        docker buildx inspect --bootstrap
    fi
    
    # Build for ARM64 (Graviton)
    docker buildx build \
        --platform linux/arm64 \
        -f Dockerfile.graviton \
        -t $IMAGE_NAME \
        --load .
    
    print_success "Docker image built for ARM64"
}

# Function to create security group
create_security_group() {
    print_status "Creating security group..."
    
    # Check if security group exists
    if aws ec2 describe-security-groups --group-names $SECURITY_GROUP --region $REGION &> /dev/null; then
        print_warning "Security group $SECURITY_GROUP already exists"
        return
    fi
    
    # Create security group
    SECURITY_GROUP_ID=$(aws ec2 create-security-group \
        --group-name $SECURITY_GROUP \
        --description "RPS Game Server Security Group" \
        --region $REGION \
        --query 'GroupId' \
        --output text)
    
    # Add rules
    aws ec2 authorize-security-group-ingress \
        --group-id $SECURITY_GROUP_ID \
        --protocol tcp \
        --port 22 \
        --cidr 0.0.0.0/0 \
        --region $REGION
    
    aws ec2 authorize-security-group-ingress \
        --group-id $SECURITY_GROUP_ID \
        --protocol tcp \
        --port 8080 \
        --cidr 0.0.0.0/0 \
        --region $REGION
    
    aws ec2 authorize-security-group-ingress \
        --group-id $SECURITY_GROUP_ID \
        --protocol tcp \
        --port 443 \
        --cidr 0.0.0.0/0 \
        --region $REGION
    
    print_success "Security group created: $SECURITY_GROUP_ID"
}

# Function to create key pair
create_key_pair() {
    print_status "Checking SSH key pair..."
    
    if aws ec2 describe-key-pairs --key-names $KEY_NAME --region $REGION &> /dev/null; then
        print_warning "Key pair $KEY_NAME already exists"
        return
    fi
    
    # Create key pair
    aws ec2 create-key-pair \
        --key-name $KEY_NAME \
        --region $REGION \
        --query 'KeyMaterial' \
        --output text > ${KEY_NAME}.pem
    
    chmod 400 ${KEY_NAME}.pem
    
    print_success "Key pair created: ${KEY_NAME}.pem"
}

# Function to get latest ARM64 AMI
get_arm64_ami() {
    print_status "Finding latest ARM64 AMI..."
    
    AMI_ID=$(aws ec2 describe-images \
        --owners amazon \
        --filters \
            "Name=name,Values=amzn2-ami-hvm-*-arm64-gp2" \
            "Name=state,Values=available" \
        --query 'Images | sort_by(@, &CreationDate) | [-1].ImageId' \
        --output text \
        --region $REGION)
    
    print_success "Using AMI: $AMI_ID"
    echo $AMI_ID
}

# Function to create user data script
create_user_data() {
    cat > user-data.sh << 'EOF'
#!/bin/bash
yum update -y

# Install Docker
yum install -y docker
systemctl start docker
systemctl enable docker
usermod -a -G docker ec2-user

# Install Docker Compose
curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
chmod +x /usr/local/bin/docker-compose

# System optimizations for Graviton
echo 'net.core.somaxconn = 65536' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 65536' >> /etc/sysctl.conf
echo 'net.core.rmem_max = 134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max = 134217728' >> /etc/sysctl.conf
sysctl -p

# Set CPU governor to performance
echo performance > /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Increase file descriptor limits
echo '* soft nofile 65536' >> /etc/security/limits.conf
echo '* hard nofile 65536' >> /etc/security/limits.conf

# Create app directory
mkdir -p /opt/rps-server
cd /opt/rps-server

# Create docker-compose.yml for production
cat > docker-compose.yml << 'COMPOSE_EOF'
version: '3.8'
services:
  rps-server:
    image: rps-server:graviton
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - RUST_LOG=info
      - SERVER_HOST=0.0.0.0
      - SERVER_PORT=8080
    restart: unless-stopped
    deploy:
      resources:
        limits:
          memory: 6G
          cpus: '3.5'
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
COMPOSE_EOF

# Note: Docker image needs to be pushed to a registry or built on instance
echo "✅ Graviton instance setup completed!"
echo "🌐 Server will be available at: http://$(curl -s http://169.254.169.254/latest/meta-data/public-ipv4):8080"

EOF
}

# Function to launch EC2 instance
launch_instance() {
    print_status "Launching Graviton EC2 instance..."
    
    AMI_ID=$(get_arm64_ami)
    create_user_data
    
    INSTANCE_ID=$(aws ec2 run-instances \
        --image-id $AMI_ID \
        --count 1 \
        --instance-type $INSTANCE_TYPE \
        --key-name $KEY_NAME \
        --security-groups $SECURITY_GROUP \
        --user-data file://user-data.sh \
        --region $REGION \
        --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=RPS-Graviton-Server},{Key=Type,Value=GameServer},{Key=Arch,Value=ARM64}]' \
        --query 'Instances[0].InstanceId' \
        --output text)
    
    print_success "Instance launched: $INSTANCE_ID"
    
    # Wait for instance to be running
    print_status "Waiting for instance to be running..."
    aws ec2 wait instance-running --instance-ids $INSTANCE_ID --region $REGION
    
    # Get public IP
    PUBLIC_IP=$(aws ec2 describe-instances \
        --instance-ids $INSTANCE_ID \
        --region $REGION \
        --query 'Reservations[0].Instances[0].PublicIpAddress' \
        --output text)
    
    print_success "Instance is running at: $PUBLIC_IP"
    
    echo ""
    echo "🎮 RPS Graviton Server Deployment Complete!"
    echo "=========================================="
    echo "Instance ID: $INSTANCE_ID"
    echo "Public IP: $PUBLIC_IP"
    echo "Instance Type: $INSTANCE_TYPE (ARM64)"
    echo "SSH Command: ssh -i ${KEY_NAME}.pem ec2-user@$PUBLIC_IP"
    echo "Server URL: http://$PUBLIC_IP:8080"
    echo "Health Check: curl http://$PUBLIC_IP:8080/health"
    echo ""
    echo "⏳ Note: Server may take 2-3 minutes to fully initialize"
    echo "📊 Monitor with: aws ec2 describe-instances --instance-ids $INSTANCE_ID"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --region REGION          AWS region (default: us-east-1)"
    echo "  --instance-type TYPE     Instance type (default: c7g.xlarge)"
    echo "  --key-name NAME          SSH key name (default: rps-server-key)"
    echo "  --security-group NAME    Security group name (default: rps-server-sg)"
    echo "  --build-only            Only build Docker image"
    echo "  --deploy-only           Only deploy (skip image build)"
    echo "  --help                  Show this help"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Full deployment"
    echo "  $0 --instance-type c7g.2xlarge       # Use larger instance"
    echo "  $0 --region eu-west-1                # Deploy to Europe"
    echo "  $0 --build-only                      # Only build image"
}

# Parse command line arguments
BUILD_ONLY=false
DEPLOY_ONLY=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --region)
            REGION="$2"
            shift 2
            ;;
        --instance-type)
            INSTANCE_TYPE="$2"
            shift 2
            ;;
        --key-name)
            KEY_NAME="$2"
            shift 2
            ;;
        --security-group)
            SECURITY_GROUP="$2"
            shift 2
            ;;
        --build-only)
            BUILD_ONLY=true
            shift
            ;;
        --deploy-only)
            DEPLOY_ONLY=true
            shift
            ;;
        --help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    print_status "Starting AWS Graviton deployment..."
    print_status "Region: $REGION"
    print_status "Instance Type: $INSTANCE_TYPE"
    
    check_prerequisites
    
    if [[ "$DEPLOY_ONLY" != true ]]; then
        build_image
    fi
    
    if [[ "$BUILD_ONLY" != true ]]; then
        create_security_group
        create_key_pair
        launch_instance
    fi
    
    if [[ "$BUILD_ONLY" == true ]]; then
        print_success "Docker image build completed!"
        print_status "To deploy: $0 --deploy-only"
    fi
}

# Run main function
main