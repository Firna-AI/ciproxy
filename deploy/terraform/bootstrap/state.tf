locals {
  state_scopes = toset(["trust", "shared", "main-preview", "production"])
  readers = merge(
    { for pair in setproduct(local.state_scopes, setunion(local.plan_accounts, ["trust_preview", "trust_production"])) : "${pair[0]}:${pair[1]}" => pair },
    { for pair in setproduct(local.state_scopes, ["shared_preview", "shared_production"]) : "${pair[0]}:${pair[1]}" => pair },
    { for pair in setproduct(local.environments, ["plan", "apply", "deploy"]) : "${pair[0]}:${pair[0]}-${pair[1]}" => [pair[0], "${pair[0]}-${pair[1]}"] },
    { for pair in setproduct(local.environments, ["plan", "apply", "deploy"]) : "shared:${pair[0]}-${pair[1]}" => ["shared", "${pair[0]}-${pair[1]}"] },
  )
  writers = merge(
    { for account in ["trust_preview", "trust_production"] : "trust:${account}" => ["trust", account] },
    { for account in local.shared_accounts : "shared:${account}" => ["shared", account] },
    { for env in local.environments : "${env}:${env}-apply" => [env, "${env}-apply"] },
  )
}
resource "google_storage_bucket_iam_member" "reader" {
  for_each = local.readers
  bucket   = "ciproxy-${each.value[0]}-tfstate-${var.project_id}"
  role     = "roles/storage.objectViewer"
  member   = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
resource "google_storage_bucket_iam_member" "writer" {
  for_each = local.writers
  bucket   = "ciproxy-${each.value[0]}-tfstate-${var.project_id}"
  role     = "roles/storage.objectAdmin"
  member   = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
# Seed owns the state buckets. Trust has bucket metadata access only here.
resource "google_storage_bucket_iam_member" "trust" {
  for_each = { for pair in setproduct(local.state_scopes, ["trust_preview", "trust_production"]) : "${pair[0]}:${pair[1]}" => pair }
  bucket   = "ciproxy-${each.value[0]}-tfstate-${var.project_id}"
  role     = "roles/storage.admin"
  member   = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
resource "google_project_iam_custom_role" "plan_lock" {
  role_id     = "ciproxyPlanLock"
  title       = "ciproxy exact Terraform locks"
  permissions = ["storage.objects.create", "storage.objects.delete"]
}
resource "google_storage_bucket_iam_member" "plan_lock" {
  for_each = merge(
    { for pair in setproduct(local.state_scopes, local.plan_accounts) : "${pair[0]}:${pair[1]}" => pair },
    { for env in local.environments : "${env}:${env}-plan" => [env, "${env}-plan"] },
  )
  bucket = "ciproxy-${each.value[0]}-tfstate-${var.project_id}"
  role   = google_project_iam_custom_role.plan_lock.name
  member = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
  condition {
    title = "exact-terraform-locks"
    expression = join(" || ", [for root in(each.value[0] == "trust" ? ["seed", "bootstrap"] : each.value[0] == "shared" ? ["shared", "edge"] : ["foundation"]) :
      "resource.name == 'projects/_/buckets/ciproxy-${each.value[0]}-tfstate-${var.project_id}/objects/${each.value[0]}/${root}/default.tflock'"
    ])
  }
}
