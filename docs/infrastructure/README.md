# Infrastructure Documentation

This document describes the infrastructure setup for the Scraper project. We maintain two environments: Development and Production.

## Overview

### Common Infrastructure
- **Region**: us-east-1 (N. Virginia)
- **State Management**: 
  - S3 bucket for Terraform state
  - DynamoDB for state locking
  - Encrypted at rest
  - Versioning enabled

## Development Environment

### Cost Optimization
Development environment is designed to be as cost-effective as possible, utilizing AWS free tier where available.

### Components

#### Networking
- Single VPC (10.0.0.0/16)
- Single Availability Zone
- Single Public Subnet (10.0.101.0/24)
- No private subnets (cost optimization)
- Internet Gateway
- No NAT Gateway (cost optimization)

#### Compute
- Single t2.micro EC2 instance (free tier eligible)
- Ubuntu 22.04 LTS
- Docker installed
- Public IP assigned
- Located in public subnet

#### Services
1. **Redis**
   - Running as Docker container
   - Port: 6379
   - No persistence
   - Single node
   - No authentication (dev only)

2. **RabbitMQ**
   - Running as Docker container
   - AMQP Port: 5672
   - Management Port: 15672
   - Default credentials:
     - Username: scraper_app
     - Password: development_only
   - Management interface enabled

#### Security
- Security Groups:
  - Redis access (6379)
  - RabbitMQ access (5672, 15672)
  - SSH access (22) - IP restricted
- All services in public subnet but IP restricted
- No sensitive data stored

### Deployment

1. **Prerequisites**
   ```bash
   # Install Terraform
   # Configure AWS credentials
   aws configure
   ```

2. **Initialize Backend**
   ```bash
   cd terraform/bootstrap
   terraform init
   terraform apply
   ```

3. **Deploy Development Environment**
   ```bash
   cd terraform/environments/dev
   cp terraform.tfvars.example terraform.tfvars
   # Edit terraform.tfvars with your IP
   terraform init
   terraform apply
   ```

### Access Information
- Redis: `<instance-ip>:6379`
- RabbitMQ: `<instance-ip>:5672`
- RabbitMQ Management: `http://<instance-ip>:15672`
- SSH: `ssh ubuntu@<instance-ip>`

## Production Environment

### Design Philosophy
Production environment is designed for reliability and security while maintaining cost-effectiveness.

### Components

#### Networking
- VPC (10.1.0.0/16)
- Two Availability Zones
- Two Private Subnets (10.1.1.0/24, 10.1.2.0/24)
- Two Public Subnets (10.1.101.0/24, 10.1.102.0/24)
- Internet Gateway
- NAT Gateway
- VPC Endpoints for AWS services

#### Services

1. **Redis (ElastiCache)**
   - Instance Type: cache.t3.small
   - Multi-AZ enabled
   - Automatic failover
   - Encryption at rest
   - Backup enabled
   - Located in private subnets

2. **RabbitMQ (Amazon MQ)**
   - Instance Type: mq.t3.small
   - Multi-AZ deployment
   - Automatic minor version upgrades
   - Maintenance window: Sunday 03:00 UTC
   - Located in private subnets
   - Credentials in AWS Secrets Manager

#### Security
- All services in private subnets
- Security groups with minimal access
- Encryption at rest
- Encryption in transit
- No direct public access

### Monitoring
- CloudWatch metrics enabled
- Log groups for services
- Enhanced monitoring for critical components

### Backup and Recovery
- Daily Redis snapshots
- 7-day retention period
- Automated backups
- Point-in-time recovery capability

## Cost Comparison

### Development
- EC2 (t2.micro): Free tier eligible
- Storage (8GB): Free tier eligible
- Data Transfer: Minimal
- **Total**: ~$0-10/month

### Production
- ElastiCache: ~$52/month
- RabbitMQ: ~$200/month
- NAT Gateway: ~$64/month
- Data Transfer: ~$20-30/month
- **Total**: ~$380-400/month

## Future Considerations
1. Consider self-hosted RabbitMQ in production for cost savings
2. Implement auto-shutdown for dev environment during non-working hours
3. Add monitoring and alerting
4. Implement disaster recovery procedures 