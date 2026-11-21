locals {
  state_scopes = toset(["trust", "shared", "main-preview", "production"])
  state_paths = {
    trust        = ["trust/seed", "trust/bootstrap"]
    shared       = ["shared/shared"]
    main-preview = ["main-preview/data", "main-preview/edge", "main-preview/foundation"]
    production   = ["production/data", "production/edge", "production/foundation"]
  }
  readers = merge(
    { for account in ["trust_preview", "trust_production"] : "trust:${account}" => { scope = "trust", account = account, paths = local.state_paths.trust } },
    { for pair in setproduct(["trust", "shared", "main-preview"], ["shared_plan", "shared_preview"]) : "${pair[0]}:${pair[1]}" => { scope = pair[0], account = pair[1], paths = pair[0] == "main-preview" ? ["main-preview/data", "main-preview/edge"] : local.state_paths[pair[0]] } },
    { for pair in setproduct(local.state_scopes, ["shared_plan_production", "shared_production"]) : "${pair[0]}:${pair[1]}" => { scope = pair[0], account = pair[1], paths = pair[0] == "main-preview" || pair[0] == "production" ? ["${pair[0]}/data", "${pair[0]}/edge"] : local.state_paths[pair[0]] } },
    { for pair in setproduct(local.environments, ["plan", "apply", "deploy"]) : "${pair[0]}:${pair[0]}-${pair[1]}" => { scope = pair[0], account = "${pair[0]}-${pair[1]}", paths = pair[1] == "deploy" ? ["${pair[0]}/data", "${pair[0]}/edge"] : ["${pair[0]}/foundation"] } },
  )
  writers = merge(
    { for account in ["trust_preview", "trust_production"] : "trust:${account}" => { scope = "trust", account = account, paths = local.state_paths.trust } },
    { for account in local.shared_accounts : "shared:${account}" => { scope = "shared", account = account, paths = local.state_paths.shared } },
    { for account in local.shared_accounts : "main-preview:${account}" => { scope = "main-preview", account = account, paths = ["main-preview/data", "main-preview/edge"] } },
    { "production:shared_production" = { scope = "production", account = "shared_production", paths = ["production/data", "production/edge"] } },
    { for env in local.environments : "${env}:${env}-apply" => { scope = env, account = "${env}-apply", paths = ["${env}/foundation"] } },
  )
}
resource "google_storage_bucket_iam_member" "reader" {
  for_each = local.readers
  bucket   = "ciproxy-${each.value.scope}-tfstate-${var.project_id}"
  role     = "roles/storage.objectViewer"
  member   = "serviceAccount:${google_service_account.ci[each.value.account].email}"
  condition {
    title = "owned-state-reads"
    expression = join(" || ", [for path in each.value.paths :
      "resource.name.startsWith('projects/_/buckets/ciproxy-${each.value.scope}-tfstate-${var.project_id}/objects/${path}/')"
    ])
  }
}
# GCS backend workspace discovery lists object names at bucket scope. It does
# not require unconditioned object payload reads or writes.
resource "google_project_iam_custom_role" "state_list" {
  role_id     = "ciproxyStateWorkspaceList"
  title       = "ciproxy Terraform workspace object names"
  permissions = ["storage.objects.list"]
}
resource "google_storage_bucket_iam_member" "state_list" {
  for_each = local.readers
  bucket   = "ciproxy-${each.value.scope}-tfstate-${var.project_id}"
  role     = google_project_iam_custom_role.state_list.name
  member   = "serviceAccount:${google_service_account.ci[each.value.account].email}"
}
resource "google_storage_bucket_iam_member" "writer" {
  for_each = local.writers
  bucket   = "ciproxy-${each.value.scope}-tfstate-${var.project_id}"
  role     = "roles/storage.objectAdmin"
  member   = "serviceAccount:${google_service_account.ci[each.value.account].email}"
  condition {
    title = "owned-state-prefixes"
    expression = join(" || ", [for path in each.value.paths :
      "resource.name.startsWith('projects/_/buckets/ciproxy-${each.value.scope}-tfstate-${var.project_id}/objects/${path}/')"
    ])
  }
}
# Main-only trust can maintain bucket configuration and access. It is already
# owner-equivalent, but receives no broad object-admin binding on data buckets.
resource "google_storage_bucket_iam_member" "trust" {
  for_each = { for pair in setproduct(local.state_scopes, ["trust_preview", "trust_production"]) : "${pair[0]}:${pair[1]}" => pair }
  bucket   = "ciproxy-${each.value[0]}-tfstate-${var.project_id}"
  role     = google_project_iam_custom_role.bucket_metadata.name
  member   = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
resource "google_project_iam_custom_role" "plan_lock" {
  role_id     = "ciproxyPlanLock"
  title       = "ciproxy exact Terraform locks"
  permissions = ["storage.objects.create", "storage.objects.delete"]
}
resource "google_storage_bucket_iam_member" "plan_lock" {
  for_each = merge(
    { for pair in setproduct(["trust", "shared", "main-preview"], ["shared_plan"]) : "${pair[0]}:${pair[1]}" => { scope = pair[0], account = pair[1], paths = pair[0] == "main-preview" ? ["main-preview/data", "main-preview/edge"] : local.state_paths[pair[0]] } },
    { for scope in local.state_scopes : "${scope}:shared_plan_production" => { scope = scope, account = "shared_plan_production", paths = scope == "main-preview" || scope == "production" ? ["${scope}/data", "${scope}/edge"] : local.state_paths[scope] } },
    { for env in local.environments : "${env}:${env}-plan" => { scope = env, account = "${env}-plan", paths = ["${env}/foundation"] } },
  )
  bucket = "ciproxy-${each.value.scope}-tfstate-${var.project_id}"
  role   = google_project_iam_custom_role.plan_lock.name
  member = "serviceAccount:${google_service_account.ci[each.value.account].email}"
  condition {
    title = "exact-terraform-locks"
    expression = join(" || ", [for path in each.value.paths :
      "resource.name == 'projects/_/buckets/ciproxy-${each.value.scope}-tfstate-${var.project_id}/objects/${path}/default.tflock'"
    ])
  }
}
