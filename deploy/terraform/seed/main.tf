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

variable "project_id" {
  type = string
}

variable "environment" {
  type = string
}

provider "google" {
  project = var.project_id
  region  = "europe-west1"
}

# bootstrap.py first uses a local-backend copy, then migrates that state to
# this bucket. Subsequent plans and applies use this remote backend in CI.
resource "google_storage_bucket" "state" {
  name                        = "ciproxy-${var.environment}-tfstate-${var.project_id}"
  location                    = "europe-west1"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false
  versioning {
    enabled = true
  }
  lifecycle {
    prevent_destroy = true
  }
}

output "state_bucket" {
  value = google_storage_bucket.state.name
}
