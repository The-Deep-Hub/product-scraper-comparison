output "vpc_id" {
  description = "The ID of the VPC"
  value       = module.networking.vpc_id
}

output "public_subnet_ids" {
  description = "List of public subnet IDs"
  value       = module.networking.public_subnet_ids
}

output "dev_instance_public_ip" {
  description = "Public IP of the dev services instance"
  value       = aws_instance.dev_services.public_ip
}

output "dev_instance_id" {
  description = "ID of the dev services instance"
  value       = aws_instance.dev_services.id
}

output "redis_endpoint" {
  description = "Redis endpoint (dev instance)"
  value       = "${aws_instance.dev_services.public_ip}:6379"
}

output "rabbitmq_endpoint" {
  description = "RabbitMQ endpoint (dev instance)"
  value       = "${aws_instance.dev_services.public_ip}:5672"
}

output "rabbitmq_management_url" {
  description = "RabbitMQ management interface URL"
  value       = "http://${aws_instance.dev_services.public_ip}:15672"
}
