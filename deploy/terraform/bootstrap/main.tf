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

locals {
  state_bucket = "ciproxy-${var.environment}-tfstate-${var.project_id}"
  # IDs are immutable. A renamed or re-created repository cannot inherit trust.
  repository_id = "1442045837"
  owner_id      = "307519338"
  ref_condition = var.environment == "production" ? "assertion.ref.startsWith('refs/tags/deploy-')" : "assertion.ref in ['refs/heads/calummoore/ci-proxy-load-balancer', 'refs/heads/main']"
  workflow      = "Firna-AI/ciproxy/.github/workflows/deploy-${var.environment}.yml@"
}

resource "google_project_service" "identity" {
  for_each = toset([
    "cloudresourcemanager.googleapis.com", "iam.googleapis.com",
    "iamcredentials.googleapis.com", "sts.googleapis.com",
  ])
  service            = each.value
  disable_on_destroy = false
}

resource "google_service_account" "deploy" {
  account_id   = "ciproxy-${var.environment}-deploy"
  display_name = "ciproxy ${var.environment} deployment"
  depends_on   = [google_project_service.identity]
}

resource "google_iam_workload_identity_pool" "github" {
  workload_identity_pool_id = "ciproxy-${var.environment}"
  display_name              = "ciproxy ${var.environment} GitHub"
  depends_on                = [google_project_service.identity]
}

resource "google_iam_workload_identity_pool_provider" "github" {
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github"
  attribute_mapping = {
    "google.subject"                = "assertion.sub"
    "attribute.repository_id"       = "assertion.repository_id"
    "attribute.repository_owner_id" = "assertion.repository_owner_id"
    "attribute.ref"                 = "assertion.ref"
    "attribute.environment"         = "assertion.environment"
    "attribute.event_name"          = "assertion.event_name"
    "attribute.workflow_ref"        = "assertion.workflow_ref"
  }
  attribute_condition = join(" && ", [
    "assertion.repository_id == '${local.repository_id}'",
    "assertion.repository_owner_id == '${local.owner_id}'",
    "assertion.environment == '${var.environment}'",
    "assertion.event_name == 'push'",
    "assertion.workflow_ref.startsWith('${local.workflow}')",
    local.ref_condition,
  ])
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

resource "google_service_account_iam_member" "federation" {
  service_account_id = google_service_account.deploy.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.repository_id/${local.repository_id}"
}

# Foundation resources stay under the local operator's credentials. CI can
# deploy Kubernetes workloads and manage only the public edge infrastructure.
resource "google_project_iam_member" "deploy" {
  for_each = toset([
    "roles/compute.loadBalancerAdmin", "roles/compute.networkViewer",
    "roles/container.developer", "roles/dns.admin", "roles/monitoring.editor",
    "roles/serviceusage.serviceUsageConsumer",
  ])
  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.deploy.email}"
}

resource "google_storage_bucket_iam_member" "state_reader" {
  bucket = local.state_bucket
  role   = "roles/storage.objectViewer"
  member = "serviceAccount:${google_service_account.deploy.email}"
}

resource "google_storage_bucket_iam_member" "edge_state_writer" {
  bucket = local.state_bucket
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.deploy.email}"
  condition {
    title       = "edge-state-only"
    description = "Allow the edge state and its Terraform lock."
    expression  = "resource.name.startsWith('projects/_/buckets/${local.state_bucket}/objects/${var.environment}/edge/')"
  }
}

output "github_environment_variables" {
  value = {
    GCP_WIF_PROVIDER           = google_iam_workload_identity_pool_provider.github.name
    GCP_DEPLOY_SERVICE_ACCOUNT = google_service_account.deploy.email
  }
}
