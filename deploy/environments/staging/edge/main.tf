terraform {
  required_version = ">= 1.17.1, < 2.0.0"
  backend "gcs" {}
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = "7.12.0"
    }
  }
}

provider "google" {
  project = "internal-498318"
  region  = "europe-west1"
}

data "terraform_remote_state" "foundation" {
  backend = "gcs"
  config = {
    bucket = "ciproxy-staging-tfstate-internal-498318"
    prefix = "staging/foundation"
  }
}

variable "control_plane_enabled" {
  type    = bool
  default = false
}

variable "enable_uptime_checks" {
  type    = bool
  default = false
}

module "edge" {
  source                = "../../../terraform/modules/edge"
  configuration         = data.terraform_remote_state.foundation.outputs.edge
  control_plane_enabled = var.control_plane_enabled
  enable_uptime_checks  = var.enable_uptime_checks
}

output "public_ip" {
  value = module.edge.public_ip
}

output "url_map" {
  value = module.edge.url_map
}

output "image_backend" {
  value = module.edge.image_backend
}
