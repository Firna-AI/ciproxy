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

data "terraform_remote_state" "shared" {
  backend = "gcs"
  config  = { bucket = "ciproxy-shared-tfstate-ciproxy", prefix = "shared/shared" }
}
# Only environments whose foundation exists are selected. Production is not
# present before the operator disables the prelaunch shared credentials.
data "terraform_remote_state" "environment" {
  for_each = data.terraform_remote_state.shared.outputs.environments
  backend  = "gcs"
  config = {
    bucket = "ciproxy-${each.key}-tfstate-ciproxy"
    prefix = "${each.key}/foundation"
  }
}
module "edge" {
  for_each = data.terraform_remote_state.shared.outputs.environments
  source   = "../modules/edge"
  configuration = merge(each.value.edge, {
    allowed_zones         = data.terraform_remote_state.environment[each.key].outputs.allowed_zones
    notification_channels = [for key, channel in google_monitoring_notification_channel.email : channel.name if startswith(key, "${each.key}:")]
  })
  control_plane_enabled = data.terraform_remote_state.environment[each.key].outputs.runtime_flags.control_plane_enabled
  enable_uptime_checks  = data.terraform_remote_state.environment[each.key].outputs.runtime_flags.enable_uptime_checks
}
output "environments" {
  value = { for env, edge in module.edge : env => {
    public_ip     = edge.public_ip
    url_map       = edge.url_map
    image_backend = edge.image_backend
    readiness     = edge.readiness
  } }
}

data "terraform_remote_state" "trust" {
  backend = "gcs"
  config  = { bucket = "ciproxy-trust-tfstate-ciproxy", prefix = "trust/bootstrap" }
}
resource "google_tags_tag_binding" "edge" {
  for_each  = module.edge
  parent    = "//compute.googleapis.com/projects/${data.terraform_remote_state.trust.outputs.project_number}/global/urlMaps/${each.value.url_map}"
  tag_value = data.terraform_remote_state.trust.outputs.edge_tags[each.key]
}

locals {
  notifications = merge([for env, state in data.terraform_remote_state.environment : {
    for email in state.outputs.notification_emails : "${env}:${email}" => { environment = env, email = email }
  }]...)
}
resource "google_monitoring_notification_channel" "email" {
  for_each     = local.notifications
  display_name = "ciproxy ${each.value.environment}: ${each.value.email}"
  type         = "email"
  labels       = { email_address = each.value.email }
}
