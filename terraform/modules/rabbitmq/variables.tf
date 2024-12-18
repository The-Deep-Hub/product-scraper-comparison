variable "project" {
  description = "Project name"
  type        = string
}

variable "environment" {
  description = "Environment name"
  type        = string
}

variable "vpc_id" {
  description = "VPC ID where RabbitMQ will be deployed"
  type        = string
}

variable "subnet_ids" {
  description = "Subnet IDs where RabbitMQ will be deployed"
  type        = list(string)
}

variable "instance_type" {
  description = "RabbitMQ broker instance type"
  type        = string
}

variable "multi_az" {
  description = "Whether to enable Multi-AZ deployment"
  type        = bool
  default     = false
}

variable "tags" {
  description = "Additional tags"
  type        = map(string)
  default     = {}
} 