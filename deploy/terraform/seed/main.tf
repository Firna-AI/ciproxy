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


provider "google" {
  project = var.project_id
  region  = "europe-west1"
}

# The complete seed apply finishes before any bootstrap resource starts.
# Keep this block in main.tf so the initial local copy enables these APIs.
resource "google_project_service" "identity" {
  for_each = toset([
    "cloudresourcemanager.googleapis.com", "iam.googleapis.com",
    "iamcredentials.googleapis.com", "sts.googleapis.com",
  ])
  service            = each.value
  disable_on_destroy = false
}

# bootstrap.py first uses a local-backend copy, then migrates that state to
# this bucket. Subsequent plans and applies use this remote backend in CI.
resource "google_storage_bucket" "state" {
  for_each                    = toset(["trust", "shared", "main-preview", "production"])
  name                        = "ciproxy-${each.key}-tfstate-${var.project_id}"
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
  value = { for scope, bucket in google_storage_bucket.state : scope => bucket.name }
}
