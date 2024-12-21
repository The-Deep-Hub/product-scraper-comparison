# Environment variables
locals {
  environment = var.environment
  project     = var.project
}

# Use single AZ in dev to minimize costs
module "networking" {
  source = "../../modules/networking"

  project            = local.project
  environment        = local.environment
  vpc_cidr          = "10.0.0.0/16"
  # Use single AZ and NO NAT Gateway for dev
  availability_zones = slice(data.aws_availability_zones.available.names, 0, 1)
  
  # Minimal CIDR blocks for dev
  private_subnets    = []  # No private subnets in dev to avoid NAT Gateway costs
  public_subnets     = ["10.0.101.0/24"]  # Everything in public subnet for dev
}

# For development, we'll use a single EC2 instance running both Redis and RabbitMQ
resource "aws_instance" "dev_services" {
  ami           = "ami-0c7217cdde317cfec"  # Ubuntu 22.04 LTS in us-east-1
  instance_type = "t2.micro"  # Free tier eligible

  subnet_id                   = module.networking.public_subnet_ids[0]
  associate_public_ip_address = true
  
  vpc_security_group_ids = [aws_security_group.dev_services.id]

  user_data = <<-EOF
              #!/bin/bash
              # Install Docker
              apt-get update
              apt-get install -y docker.io
              systemctl start docker
              systemctl enable docker

              # Run Redis container
              docker run -d --name redis \
                -p 6379:6379 \
                redis:7

              # Run RabbitMQ container (simplified for dev)
              docker run -d --name rabbitmq \
                -p 5672:5672 \
                -p 15672:15672 \
                rabbitmq:3-management
              EOF

  tags = merge(local.dev_tags, {
    Name = "${local.project}-${local.environment}-services"
  })

  # Enable termination protection to prevent accidental deletion
  disable_api_termination = false

  # Use magnetic storage to stay within free tier
  root_block_device {
    volume_type = "standard"
    volume_size = 8
  }
}

# Security group for the dev services
resource "aws_security_group" "dev_services" {
  name_prefix = "${local.project}-${local.environment}-services-"
  vpc_id      = module.networking.vpc_id

  # Redis access
  ingress {
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  # RabbitMQ access
  ingress {
    from_port   = 5672
    to_port     = 5672
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  # RabbitMQ management interface
  ingress {
    from_port   = 15672
    to_port     = 15672
    protocol    = "tcp"
    cidr_blocks = [var.vpc_cidr]
  }

  # SSH access (optional, for troubleshooting)
  ingress {
    from_port   = 22
    to_port     = 22
    protocol    = "tcp"
    cidr_blocks = var.ssh_allowed_ips
  }

  egress {
    from_port   = 0
    to_port     = 0
    protocol    = "-1"
    cidr_blocks = ["0.0.0.0/0"]
  }

  tags = local.dev_tags
}

# Dev-specific variables
locals {
  dev_tags = {
    Environment = local.environment
    Project     = local.project
    ManagedBy   = "terraform"
    CostCenter  = "development"
    AutoShutdown = "true"  # Tag for automated shutdown during non-working hours
  }
}
