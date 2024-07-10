terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "5.56.1"
    }
  }
  backend "s3" {
    bucket = "nicks-terraform-states"
    region = "ap-southeast-2"
  }
}

provider "aws" {
  region = local.region
  default_tags {
    tags = local.tags
  }
}

data "aws_caller_identity" "identity" {}

locals {
  region                    = "eu-west-2"
  prefix                    = "RustLambda-${title(var.environment)}"
  prefix_lower              = "rust-lambda-${lower(var.environment)}"
  prefix_parameter          = "/RustLambda/${title(var.environment)}"
  aws_account_id            = data.aws_caller_identity.identity.account_id
  automated_tester_username = "nick.dave.sullivan+testing@gmail.com"
  root_dir                  = "${path.root}/.."
  lambda_dir                = "${local.root_dir}/lambda"

  lambda_names = {
    "hello"   = "${local.prefix}-Hello"
    "goodbye" = "${local.prefix}-Goodbye"
  }
  tags = {
    Project     = "Rust Lambda"
    Environment = var.environment
  }
}
