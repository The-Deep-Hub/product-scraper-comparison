locals {
  name = "${var.project}-${var.environment}-redis"
  tags = merge(var.tags, {
    Name        = local.name
    Project     = var.project
    Environment = var.environment
    ManagedBy   = "terraform"
  })
}

resource "aws_elasticache_subnet_group" "redis" {
  name       = "${local.name}-subnet-group"
  subnet_ids = var.subnet_ids
}

resource "aws_security_group" "redis" {
  name_prefix = "${local.name}-sg-"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 6379
    to_port     = 6379
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.selected.cidr_block]
  }

  tags = local.tags
}

resource "aws_elasticache_parameter_group" "redis" {
  family = "redis7"
  name   = "${local.name}-params"

  parameter {
    name  = "maxmemory-policy"
    value = "allkeys-lru"
  }
}

resource "aws_elasticache_replication_group" "redis" {
  replication_group_id = replace(local.name, "-", "")
  description         = "Redis cluster for ${var.project} ${var.environment}"
  
  node_type                  = var.node_type
  port                      = 6379
  parameter_group_name      = aws_elasticache_parameter_group.redis.name
  subnet_group_name         = aws_elasticache_subnet_group.redis.name
  security_group_ids        = [aws_security_group.redis.id]
  
  # Single node for dev, multi-node for prod
  num_cache_clusters        = var.multi_az ? 2 : 1
  automatic_failover_enabled = var.multi_az

  # Cost optimization settings
  snapshot_retention_limit  = var.environment == "prod" ? 7 : 0
  snapshot_window          = var.environment == "prod" ? "03:00-04:00" : null
  maintenance_window       = "sun:05:00-sun:06:00"

  tags = local.tags
}

data "aws_vpc" "selected" {
  id = var.vpc_id
} 