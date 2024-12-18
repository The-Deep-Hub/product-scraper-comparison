variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "project" {
  description = "Project name"
  type        = string
  default     = "scraper"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
}

variable "ssh_allowed_ips" {
  description = "List of IPs allowed to SSH into the dev instance"
  type        = list(string)
  default     = []  # Should be provided in terraform.tfvars
}

variable "dev_instance_type" {
  description = "Instance type for dev services"
  type        = string
  default     = "t2.micro"  # Free tier eligible
}
