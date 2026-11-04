variable "project_id" {
  type = string
}

variable "environment" {
  type = string
}

variable "domain" {
  type = string
}

variable "notification_emails" {
  description = "Operator addresses that receive public uptime alerts."
  type        = set(string)
}

locals {
  region        = "europe-west1"
  allowed_zones = ["europe-west1-b", "europe-west1-c", "europe-west1-d"]
  prefix        = "ciproxy-${var.environment}"
  ci_email      = "ciproxy-${var.environment}-deploy@${var.project_id}.iam.gserviceaccount.com"
  hosts = {
    site      = var.domain
    dashboard = "app.${var.domain}"
    releases  = "get.${var.domain}"
  }
  control_plane_secrets = toset([
    "github-app-private-key", "github-webhook-secret", "github-oauth-secret",
    "stripe-secret-key", "stripe-webhook-secret", "session-key",
    "cdn-signing-key",
  ])
}

resource "google_project_service" "service" {
  for_each = toset([
    "artifactregistry.googleapis.com", "cloudkms.googleapis.com",
    "compute.googleapis.com", "container.googleapis.com", "dns.googleapis.com",
    "logging.googleapis.com", "monitoring.googleapis.com",
    "secretmanager.googleapis.com", "servicenetworking.googleapis.com",
    "sqladmin.googleapis.com", "storage.googleapis.com",
  ])
  service            = each.value
  disable_on_destroy = false
}
