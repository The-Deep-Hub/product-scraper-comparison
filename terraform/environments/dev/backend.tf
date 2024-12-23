terraform {
  backend "s3" {
    bucket         = "scraper-terraform-state"
    key            = "dev/terraform.tfstate"
    region         = "us-east-1"
    dynamodb_table = "scraper-terraform-locks"
    encrypt        = true
  }
} 