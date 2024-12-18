provider "aws" {
  region = "us-east-1"
}

locals {
  environment = "prod"
  project     = "scraper"
}

# Use 2 AZs in production for balanced HA/cost
module "networking" {
  source = "../../modules/networking"

  project            = local.project
  environment        = local.environment
  vpc_cidr          = "10.1.0.0/16"
  availability_zones = slice(data.aws_availability_zones.available.names, 0, 2)
  
  # Production CIDR blocks
  private_subnets    = ["10.1.1.0/24", "10.1.2.0/24"]
  public_subnets     = ["10.1.101.0/24", "10.1.102.0/24"]
}

# Redis for production (multi-AZ)
module "redis" {
  source = "../../modules/redis"

  project     = local.project
  environment = local.environment
  vpc_id      = module.networking.vpc_id
  subnet_ids  = module.networking.private_subnet_ids
  
  # Cost-effective production instance
  node_type   = "cache.t3.small"
  multi_az    = true
  
  tags = local.prod_tags
}

# RabbitMQ for production (multi-AZ)
module "rabbitmq" {
  source = "../../modules/rabbitmq"

  project       = local.project
  environment   = local.environment
  vpc_id        = module.networking.vpc_id
  subnet_ids    = module.networking.private_subnet_ids
  
  # Cost-effective production instance
  instance_type = "mq.t3.small"
  multi_az      = true
  
  tags = local.prod_tags
}

# Production-specific variables
locals {
  prod_tags = {
    Environment = local.environment
    Project     = local.project
    ManagedBy   = "terraform"
    CostCenter  = "production"
    Backup      = "true"
  }

  # Production-specific settings
  instance_type = "t3.small"     # Balance between cost and performance
  multi_az      = true          # Two AZ deployment for HA
  backup_enabled = true         # Enable backups for production
  monitoring_enabled = true     # Enable detailed monitoring
}
