provider "aws" {
  region = "us-east-1"
}

locals {
  environment = "staging"
  project     = "scraper"
}

# Use 2 AZs in staging - balance between reliability and cost
module "networking" {
  source = "../../modules/networking"

  project            = local.project
  environment        = local.environment
  vpc_cidr          = "10.1.0.0/16"  # Different CIDR for staging
  availability_zones = slice(data.aws_availability_zones.available.names, 0, 2)
  
  # Moderate-sized CIDR blocks
  private_subnets    = ["10.1.1.0/24", "10.1.2.0/24"]
  public_subnets     = ["10.1.101.0/24", "10.1.102.0/24"]
}

# Staging-specific variables
locals {
  staging_tags = {
    Environment = local.environment
    Project     = local.project
    ManagedBy   = "terraform"
    CostCenter  = "staging"
  }
}
