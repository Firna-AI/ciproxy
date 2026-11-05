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

variable "production_enabled" { type = bool }
locals {
  region       = "europe-west1"
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
    runtime_repository = "europe-west1-docker.pkg.dev/ciproxy/${google_artifact_registry_repository.runtime.repository_id}"
    allowed_zones      = module.zones.allowed_zones
  }
}
output "production_enabled" { value = var.production_enabled }

# GKE creates its workload identity pool first. These exact-account policy
# writes are granted in bootstrap without granting project IAM to shared apply.
resource "google_service_account_iam_member" "runtime" {
  for_each           = local.environments
  service_account_id = "projects/ciproxy/serviceAccounts/ciproxy-${each.key}-control@ciproxy.iam.gserviceaccount.com"
  role               = "roles/iam.workloadIdentityUser"
  member             = "serviceAccount:ciproxy.svc.id.goog[ciproxy-${each.key}/control-plane]"
  depends_on         = [google_container_cluster.main]
}
