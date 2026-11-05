# Workload deployment keeps its existing roles. Terraform applies use a
# separate account, restricted by the reusable workflow's OIDC claim.
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

resource "google_project_iam_member" "plan" {
  for_each = toset([
    "roles/viewer", "roles/iam.securityReviewer", "roles/iam.roleViewer",
    "roles/iam.serviceAccountViewer", "roles/iam.workloadIdentityPoolViewer",
    "roles/secretmanager.viewer", "roles/cloudkms.viewer",
  ])
  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.promotion["plan"].email}"
}

# Project IAM administration makes this account owner-equivalent. Only the
# deploy-apply.yml reusable workflow can exchange an OIDC token for it.
resource "google_project_iam_member" "apply" {
  for_each = toset([
    "roles/serviceusage.serviceUsageAdmin", "roles/resourcemanager.projectIamAdmin",
    "roles/iam.serviceAccountAdmin", "roles/iam.serviceAccountUser",
    "roles/iam.workloadIdentityPoolAdmin", "roles/iam.roleAdmin",
    "roles/storage.admin", "roles/compute.networkAdmin", "roles/compute.securityAdmin",
    "roles/compute.loadBalancerAdmin", "roles/container.admin",
    "roles/servicenetworking.networksAdmin", "roles/cloudsql.admin",
    "roles/secretmanager.admin", "roles/cloudkms.admin",
    "roles/artifactregistry.admin", "roles/dns.admin", "roles/monitoring.editor",
  ])
  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.promotion["apply"].email}"
}

resource "google_storage_bucket_iam_member" "state_reader" {
  bucket = local.state_bucket
  role   = "roles/storage.objectViewer"
  member = "serviceAccount:${google_service_account.deploy.email}"
}

resource "google_storage_bucket_iam_member" "plan_state_reader" {
  bucket = local.state_bucket
  role   = "roles/storage.objectViewer"
  member = "serviceAccount:${google_service_account.promotion["plan"].email}"
}

resource "google_project_iam_custom_role" "plan_lock" {
  role_id     = "ciproxy${replace(title(var.environment), "-", "")}PlanLock"
  title       = "ciproxy ${var.environment} Terraform locks"
  description = "Write only Terraform lock objects under the conditional bucket grant."
  permissions = ["storage.objects.create", "storage.objects.delete"]
}

resource "google_storage_bucket_iam_member" "plan_lock" {
  bucket = local.state_bucket
  role   = google_project_iam_custom_role.plan_lock.name
  member = "serviceAccount:${google_service_account.promotion["plan"].email}"
  condition {
    title = "terraform-locks-only"
    expression = join(" || ", [for root in ["seed", "bootstrap", "foundation", "edge"] :
      "resource.name == 'projects/_/buckets/${local.state_bucket}/objects/${var.environment}/${root}/default.tflock'"
    ])
  }
}

resource "google_storage_bucket_iam_member" "apply_state_writer" {
  bucket = local.state_bucket
  role   = "roles/storage.objectAdmin"
  member = "serviceAccount:${google_service_account.promotion["apply"].email}"
}
