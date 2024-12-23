output "vpc_id" {
  description = "The ID of the VPC"
  value       = module.networking.vpc_id
}

output "private_subnet_ids" {
  description = "List of private subnet IDs"
  value       = module.networking.private_subnet_ids
}

output "public_subnet_ids" {
  description = "List of public subnet IDs"
  value       = module.networking.public_subnet_ids
}

output "redis_endpoint" {
  description = "Redis primary endpoint"
  value       = module.redis.primary_endpoint
  sensitive   = true
}

output "redis_reader_endpoint" {
  description = "Redis reader endpoint"
  value       = module.redis.reader_endpoint
  sensitive   = true
}

output "rabbitmq_endpoints" {
  description = "RabbitMQ endpoints"
  value       = module.rabbitmq.endpoints
  sensitive   = true
}

output "rabbitmq_secrets_arn" {
  description = "ARN of the secrets manager secret containing RabbitMQ credentials"
  value       = module.rabbitmq.credentials_secret_arn
}
