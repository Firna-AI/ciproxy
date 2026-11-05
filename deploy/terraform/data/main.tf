terraform {
  required_version = ">= 1.17.1, < 2.0.0"
  backend "gcs" {}
  required_providers {
    google = { source = "hashicorp/google", version = "7.12.0" }
  }
}
provider "google" {
  project = "ciproxy"
  region  = "us-central1"
}

variable "environment" { type = string }
data "terraform_remote_state" "shared" {
  backend = "gcs"
  config  = { bucket = "ciproxy-shared-tfstate-ciproxy", prefix = "shared/shared" }
}
module "environment" {
  source      = "../modules/foundation"
  project_id  = "ciproxy"
  environment = var.environment
  domain      = "ciproxy.dev"
  shared      = data.terraform_remote_state.shared.outputs.platform
}
output "deployment" { value = module.environment.deployment }
output "edge" { value = module.environment.edge }
output "dns_name_servers" { value = module.environment.dns_name_servers }
