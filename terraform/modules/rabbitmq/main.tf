locals {
  name = "${var.project}-${var.environment}-rabbitmq"
  tags = merge(var.tags, {
    Name        = local.name
    Project     = var.project
    Environment = var.environment
    ManagedBy   = "terraform"
  })
}

resource "aws_security_group" "rabbitmq" {
  name_prefix = "${local.name}-sg-"
  vpc_id      = var.vpc_id

  ingress {
    from_port   = 5671
    to_port     = 5671
    protocol    = "tcp"
    cidr_blocks = [data.aws_vpc.selected.cidr_block]
  }

  tags = local.tags
}

resource "aws_mq_broker" "rabbitmq" {
  broker_name = local.name

  engine_type         = "RabbitMQ"
  engine_version      = "3.11.16"
  host_instance_type  = var.instance_type
  security_groups     = [aws_security_group.rabbitmq.id]
  subnet_ids          = var.multi_az ? var.subnet_ids : [var.subnet_ids[0]]

  deployment_mode = var.multi_az ? "CLUSTER_MULTI_AZ" : "SINGLE_INSTANCE"

  # Cost optimization settings
  auto_minor_version_upgrade = true
  publicly_accessible       = false

  maintenance_window_start_time {
    day_of_week = "SUNDAY"
    time_of_day = "03:00"
    time_zone   = "UTC"
  }

  logs {
    general = true
  }

  user {
    username = "scraper_app"
    password = random_password.rabbitmq_password.result
  }

  tags = local.tags
}

resource "random_password" "rabbitmq_password" {
  length           = 16
  special          = true
  override_special = "!#$%&*()-_=+[]{}<>:?"
}

resource "aws_secretsmanager_secret" "rabbitmq_credentials" {
  name = "${local.name}-credentials"
  tags = local.tags
}

resource "aws_secretsmanager_secret_version" "rabbitmq_credentials" {
  secret_id = aws_secretsmanager_secret.rabbitmq_credentials.id
  secret_string = jsonencode({
    username = "scraper_app"
    password = random_password.rabbitmq_password.result
    host     = aws_mq_broker.rabbitmq.instances[0].endpoints[0]
  })
}

data "aws_vpc" "selected" {
  id = var.vpc_id
} 