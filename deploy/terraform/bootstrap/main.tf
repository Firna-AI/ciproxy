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
  state_bucket   = "ciproxy-${var.environment}-tfstate-${var.project_id}"
  repository_id  = "1442045837"
  owner_id       = "307519338"
  ref_condition  = var.environment == "production" ? "assertion.ref == 'refs/heads/main'" : "assertion.ref in ['refs/heads/calummoore/ci-proxy-load-balancer', 'refs/heads/main']"
  event          = var.environment == "production" ? "workflow_run" : "push"
  workflow       = "Firna-AI/ciproxy/.github/workflows/deploy-${var.environment}.yml@"
  apply_workflow = "Firna-AI/ciproxy/.github/workflows/deploy-apply.yml@"
  attributes = {
    "google.subject"                = "assertion.sub"
    "attribute.repository_id"       = "assertion.repository_id"
    "attribute.repository_owner_id" = "assertion.repository_owner_id"
    "attribute.ref"                 = "assertion.ref"
    "attribute.environment"         = "assertion.environment"
    "attribute.event_name"          = "assertion.event_name"
    "attribute.workflow_ref"        = "assertion.workflow_ref"
  }
  trust = [
    "assertion.repository_id == '${local.repository_id}'",
    "assertion.repository_owner_id == '${local.owner_id}'",
    "assertion.environment == '${var.environment}'",
    "assertion.event_name == '${local.event}'",
    "assertion.workflow_ref == '${local.workflow}' + assertion.ref",
    local.ref_condition,
  ]
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
  display_name = "ciproxy ${var.environment} workloads"
  depends_on   = [google_project_service.identity]
}

resource "google_service_account" "promotion" {
  for_each     = toset(["plan", "apply"])
  account_id   = "ciproxy-${var.environment}-${each.key}"
  display_name = "ciproxy ${var.environment} Terraform ${each.key}"
  depends_on   = [google_project_service.identity]
}

resource "google_iam_workload_identity_pool" "github" {
  workload_identity_pool_id = "ciproxy-${var.environment}"
  display_name              = "ciproxy ${var.environment} GitHub"
  depends_on                = [google_project_service.identity]
}

# Keep the deployed provider and account addresses. Each account binds a
# distinct mapped access attribute, not every identity in the shared pool.
resource "google_iam_workload_identity_pool_provider" "github" {
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = "github"
  attribute_mapping                  = merge(local.attributes, { "attribute.access" = "'deploy'" })
  attribute_condition                = join(" && ", local.trust)
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

resource "google_iam_workload_identity_pool_provider" "promotion" {
  for_each                           = toset(["plan", "apply"])
  workload_identity_pool_id          = google_iam_workload_identity_pool.github.workload_identity_pool_id
  workload_identity_pool_provider_id = each.key
  attribute_mapping = merge(local.attributes, {
    "attribute.access" = "'${each.key}'"
  }, each.key == "apply" ? { "attribute.job_workflow_ref" = "assertion.job_workflow_ref" } : {})
  attribute_condition = join(" && ", concat(local.trust, each.key == "apply" ? [
    "assertion.job_workflow_ref == '${local.apply_workflow}' + assertion.ref",
  ] : []))
  oidc {
    issuer_uri = "https://token.actions.githubusercontent.com"
  }
}

resource "google_service_account_iam_member" "federation" {
  service_account_id = google_service_account.deploy.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.access/deploy"
}

resource "google_service_account_iam_member" "promotion" {
  for_each           = google_service_account.promotion
  service_account_id = each.value.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github.name}/attribute.access/${each.key}"
}

output "github_environment_variables" {
  value = {
    GCP_WIF_PROVIDER           = google_iam_workload_identity_pool_provider.github.name
    GCP_DEPLOY_SERVICE_ACCOUNT = google_service_account.deploy.email
    GCP_PLAN_WIF_PROVIDER      = google_iam_workload_identity_pool_provider.promotion["plan"].name
    GCP_PLAN_SERVICE_ACCOUNT   = google_service_account.promotion["plan"].email
    GCP_APPLY_WIF_PROVIDER     = google_iam_workload_identity_pool_provider.promotion["apply"].name
    GCP_APPLY_SERVICE_ACCOUNT  = google_service_account.promotion["apply"].email
  }
}
