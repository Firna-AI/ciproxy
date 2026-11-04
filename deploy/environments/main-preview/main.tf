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

variable "notification_emails" {
  type        = set(string)
  description = "Operator addresses for uptime alerts."
}

module "foundation" {
  source              = "../../terraform/modules/foundation"
  project_id          = "internal-498318"
  environment         = "main-preview"
  domain              = "ciproxy.dev"
  notification_emails = var.notification_emails
}

output "deployment" {
  value = module.foundation.deployment
}

output "edge" {
  value = module.foundation.edge
}

output "dns_name_servers" {
  value = module.foundation.dns_name_servers
}
