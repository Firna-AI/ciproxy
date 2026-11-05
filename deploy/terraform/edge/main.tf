terraform {
  required_version = ">= 1.17.1, < 2.0.0"
  backend "gcs" {}
  required_providers {
    google = { source = "hashicorp/google", version = "7.12.0" }
  }
}
provider "google" {
  project = "ciproxy"
  region  = "europe-west1"
}

variable "environment" { type = string }
variable "workload" {
  type = object({
    control_plane_enabled = bool
    enable_uptime_checks  = bool
    notification_emails   = set(string)
    allowed_zones         = list(string)
    source_sha            = string
  })
}
data "terraform_remote_state" "infrastructure" {
  backend = "gcs"
  config = {
    bucket = "ciproxy-${var.environment}-tfstate-ciproxy"
    prefix = "${var.environment}/data"
  }
}
data "terraform_remote_state" "trust" {
  backend = "gcs"
  config  = { bucket = "ciproxy-trust-tfstate-ciproxy", prefix = "trust/bootstrap" }
}
module "zones" { source = "../modules/zones" }
module "edge" {
  source = "../modules/edge"
  configuration = merge(data.terraform_remote_state.infrastructure.outputs.edge, {
    allowed_zones         = var.workload.allowed_zones
    notification_channels = [for channel in google_monitoring_notification_channel.email : channel.name]
  })
  zone_map              = module.zones.catalog
  project_number        = data.terraform_remote_state.trust.outputs.project_number
  environment_tag       = data.terraform_remote_state.trust.outputs.edge_tags[var.environment]
  control_plane_enabled = var.workload.control_plane_enabled
  enable_uptime_checks  = var.workload.enable_uptime_checks
}
resource "google_monitoring_notification_channel" "email" {
  for_each     = var.workload.notification_emails
  display_name = "ciproxy ${var.environment}: ${each.value}"
  type         = "email"
  labels       = { email_address = each.value }
}
output "public_ip" { value = module.edge.public_ip }
output "url_map" { value = module.edge.url_map }
output "image_backend" { value = module.edge.image_backend }
output "readiness" { value = module.edge.readiness }
