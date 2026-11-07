terraform {
  required_version = ">= 1.17.1, < 2.0.0"
  backend "gcs" {}
  required_providers {
    google = { source = "hashicorp/google", version = "7.12.0" }
  }
}
provider "google" {
  project = var.project_id
  region  = "us-central1"
}
variable "project_id" { type = string }
variable "production_enabled" { type = bool }
locals {
  environments = toset(["main-preview", "production"])
  access = merge(
    { for pair in setproduct(local.environments, ["plan", "apply", "deploy"]) :
    "${pair[0]}-${pair[1]}" => { environment = pair[0], access = pair[1], main_only = false } },
    {
      shared_plan            = { environment = "main-preview", access = "shared-plan", main_only = false }
      shared_plan_production = { environment = "production", access = "shared-plan", main_only = true }
      shared_preview         = { environment = "main-preview", access = "shared-apply", main_only = false }
      shared_production      = { environment = "production", access = "shared-apply", main_only = true }
      trust_preview          = { environment = "main-preview", access = "trust-apply", main_only = true }
      trust_production       = { environment = "production", access = "trust-apply", main_only = true }
      image_lookup           = { environment = "main-preview", access = "image-lookup", main_only = false }
      image_publish          = { environment = "main-preview", access = "image-publish", main_only = false }
    }
  )
  account_ids = {
    shared_plan            = "ciproxy-shared-plan"
    shared_plan_production = "ciproxy-shared-prod-plan"
    shared_preview         = "ciproxy-shared-preview-apply"
    shared_production      = "ciproxy-shared-prod-apply"
    trust_preview          = "ciproxy-trust-preview-apply"
    trust_production       = "ciproxy-trust-prod-apply"
    image_lookup           = "ciproxy-image-lookup"
    image_publish          = "ciproxy-image-publish"
  }
  plan_accounts  = toset(["shared_plan", "shared_plan_production"])
  retired        = toset(["shared_plan", "shared_preview", "trust_preview"])
  apply_workflow = "Firna-AI/ciproxy/.github/workflows/deploy-apply.yml@"
}
resource "google_service_account" "ci" {
  for_each   = local.access
  account_id = lookup(local.account_ids, each.key, "ciproxy-${each.key}")
  disabled   = var.production_enabled && contains(local.retired, each.key)
  lifecycle { prevent_destroy = true }
}
resource "google_service_account" "runtime" {
  for_each   = local.environments
  account_id = "ciproxy-${each.key}-control"
}
resource "google_service_account" "nodes" { account_id = "ciproxy-shared-gke" }
resource "google_iam_workload_identity_pool" "github" {
  for_each                  = local.access
  workload_identity_pool_id = "ciproxy-${replace(each.key, "_", "-")}"
  disabled                  = var.production_enabled && contains(local.retired, each.key)
}
resource "google_iam_workload_identity_pool_provider" "github" {
  for_each                           = local.access
  workload_identity_pool_id          = google_iam_workload_identity_pool.github[each.key].workload_identity_pool_id
  workload_identity_pool_provider_id = "github"
  attribute_mapping = merge({
    "google.subject"                = "assertion.sub"
    "attribute.access"              = "'${each.key}'"
    "attribute.repository_id"       = "assertion.repository_id"
    "attribute.repository_owner_id" = "assertion.repository_owner_id"
    "attribute.environment"         = "assertion.environment"
    "attribute.ref"                 = "assertion.ref"
    "attribute.event_name"          = "assertion.event_name"
    "attribute.workflow_ref"        = "assertion.workflow_ref"
  }, endswith(each.value.access, "apply") ? { "attribute.job_workflow_ref" = "assertion.job_workflow_ref" } : {})
  attribute_condition = join(" && ", concat([
    "assertion.repository_id == '1442045837'",
    "assertion.repository_owner_id == '307519338'",
    "assertion.environment == '${each.value.environment}'",
    "assertion.event_name == '${each.value.environment == "production" ? "workflow_run" : "push"}'",
    "assertion.workflow_ref == 'Firna-AI/ciproxy/.github/workflows/deploy-${each.value.environment}.yml@' + assertion.ref",
    each.value.main_only || each.value.environment == "production" ? "assertion.ref == 'refs/heads/main'" : "assertion.ref in ['refs/heads/calummoore/ci-proxy-load-balancer', 'refs/heads/main']",
    ], endswith(each.value.access, "apply") ? [
    "assertion.job_workflow_ref == '${local.apply_workflow}' + assertion.ref",
  ] : []))
  disabled = var.production_enabled && contains(local.retired, each.key)
  oidc { issuer_uri = "https://token.actions.githubusercontent.com" }
}
resource "google_service_account_iam_member" "federation" {
  for_each           = local.access
  service_account_id = google_service_account.ci[each.key].name
  role               = "roles/iam.workloadIdentityUser"
  member             = "principalSet://iam.googleapis.com/${google_iam_workload_identity_pool.github[each.key].name}/attribute.access/${each.key}"
}
resource "google_project_iam_member" "sql_runtime" {
  for_each = { for pair in setproduct(local.environments, ["roles/cloudsql.client", "roles/cloudsql.instanceUser"]) : "${pair[0]}:${pair[1]}" => pair }
  project  = var.project_id
  role     = each.value[1]
  member   = "serviceAccount:${google_service_account.runtime[each.value[0]].email}"
  condition {
    title      = "one-sql-instance"
    expression = "resource.type == 'sqladmin.googleapis.com/Instance' && resource.name == 'projects/${var.project_id}/instances/ciproxy-${each.value[0]}'"
  }
}
resource "google_project_iam_member" "nodes" {
  project = var.project_id
  role    = "roles/container.defaultNodeServiceAccount"
  member  = "serviceAccount:${google_service_account.nodes.email}"
}
output "production_enabled" { value = var.production_enabled }
output "github_environment_variables" {
  value = { for env in local.environments : env => merge({
    GCP_WIF_PROVIDER                 = google_iam_workload_identity_pool_provider.github["${env}-deploy"].name
    GCP_DEPLOY_SERVICE_ACCOUNT       = google_service_account.ci["${env}-deploy"].email
    GCP_PLAN_WIF_PROVIDER            = google_iam_workload_identity_pool_provider.github["${env}-plan"].name
    GCP_PLAN_SERVICE_ACCOUNT         = google_service_account.ci["${env}-plan"].email
    GCP_APPLY_WIF_PROVIDER           = google_iam_workload_identity_pool_provider.github["${env}-apply"].name
    GCP_APPLY_SERVICE_ACCOUNT        = google_service_account.ci["${env}-apply"].email
    GCP_SHARED_PLAN_WIF_PROVIDER     = google_iam_workload_identity_pool_provider.github[env == "production" ? "shared_plan_production" : "shared_plan"].name
    GCP_SHARED_PLAN_SERVICE_ACCOUNT  = google_service_account.ci[env == "production" ? "shared_plan_production" : "shared_plan"].email
    GCP_SHARED_APPLY_WIF_PROVIDER    = google_iam_workload_identity_pool_provider.github[env == "production" ? "shared_production" : "shared_preview"].name
    GCP_SHARED_APPLY_SERVICE_ACCOUNT = google_service_account.ci[env == "production" ? "shared_production" : "shared_preview"].email
    GCP_TRUST_APPLY_WIF_PROVIDER     = google_iam_workload_identity_pool_provider.github[env == "production" ? "trust_production" : "trust_preview"].name
    GCP_TRUST_APPLY_SERVICE_ACCOUNT  = google_service_account.ci[env == "production" ? "trust_production" : "trust_preview"].email
    }, env == "main-preview" ? {
    GCP_IMAGE_LOOKUP_WIF_PROVIDER     = google_iam_workload_identity_pool_provider.github["image_lookup"].name
    GCP_IMAGE_LOOKUP_SERVICE_ACCOUNT  = google_service_account.ci["image_lookup"].email
    GCP_IMAGE_PUBLISH_WIF_PROVIDER    = google_iam_workload_identity_pool_provider.github["image_publish"].name
    GCP_IMAGE_PUBLISH_SERVICE_ACCOUNT = google_service_account.ci["image_publish"].email
  } : {}) }
}
