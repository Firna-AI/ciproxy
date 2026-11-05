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

variable "production_enabled" { type = bool }
locals {
  region       = "us-central1"
  prefix       = "ciproxy-shared"
  node_account = "ciproxy-shared-gke@ciproxy.iam.gserviceaccount.com"
  environments = var.production_enabled ? toset(["main-preview", "production"]) : toset(["main-preview"])
}
resource "google_project_service" "service" {
  for_each = toset([
    "artifactregistry.googleapis.com", "cloudkms.googleapis.com",
    "compute.googleapis.com", "container.googleapis.com", "dns.googleapis.com",
    "logging.googleapis.com", "monitoring.googleapis.com", "secretmanager.googleapis.com",
    "servicenetworking.googleapis.com", "sqladmin.googleapis.com", "storage.googleapis.com",
  ])
  service            = each.value
  disable_on_destroy = false
}
module "zones" { source = "../modules/zones" }
output "platform" {
  value = {
    network_id         = google_compute_network.main.id
    cluster_name       = google_container_cluster.main.name
    runtime_repository = "us-central1-docker.pkg.dev/ciproxy/${google_artifact_registry_repository.runtime.repository_id}"
    allowed_zones      = module.zones.allowed_zones
  }
}
output "production_enabled" { value = var.production_enabled }

data "terraform_remote_state" "trust" {
  backend = "gcs"
  config  = { bucket = "ciproxy-trust-tfstate-ciproxy", prefix = "trust/bootstrap" }
}
# Empty trust input installs no grant during the operator's first core apply.
# Never read runtime UIDs from ready-workload evidence or preview state.
resource "google_service_account_iam_member" "runtime" {
  for_each           = { for env, uid in data.terraform_remote_state.trust.outputs.runtime_service_account_uids : env => uid if contains(local.environments, env) }
  service_account_id = "projects/ciproxy/serviceAccounts/ciproxy-${each.key}-control@ciproxy.iam.gserviceaccount.com"
  role               = "roles/iam.workloadIdentityUser"
  member             = "principal://iam.googleapis.com/projects/${data.terraform_remote_state.trust.outputs.project_number}/locations/global/workloadIdentityPools/ciproxy.svc.id.goog/kubernetes.serviceaccount.uid/${each.value}"
  depends_on         = [google_container_cluster.main]
}
