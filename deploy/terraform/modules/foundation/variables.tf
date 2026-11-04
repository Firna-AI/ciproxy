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
  host_prefix   = var.environment == "main-preview" ? "br-main.preview." : ""
  hosts = {
    site      = "${local.host_prefix}${var.domain}"
    dashboard = "${local.host_prefix}app.${var.domain}"
    releases  = "${local.host_prefix}get.${var.domain}"
  }
  dns_domains = var.environment == "main-preview" ? {
    site      = "preview.${var.domain}"
    dashboard = "preview.app.${var.domain}"
    releases  = "preview.get.${var.domain}"
  } : { site = var.domain }
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
