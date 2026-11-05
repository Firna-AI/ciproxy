# These permissions open the DNS endpoint, without authorizing Kubernetes
# objects. Namespace RoleBindings are installed once by the operator.
resource "google_project_iam_custom_role" "cluster_access" {
  role_id     = "ciproxyClusterAccess"
  title       = "ciproxy cluster connection only"
  permissions = ["container.clusters.get", "container.clusters.connect"]
}
resource "google_project_iam_member" "cluster_access" {
  for_each = local.environments
  project  = var.project_id
  role     = google_project_iam_custom_role.cluster_access.name
  member   = "serviceAccount:${google_service_account.ci["${each.key}-deploy"].email}"
}
resource "google_project_iam_member" "service_consumer" {
  for_each = toset(flatten([for env in local.environments : ["${env}-plan", "${env}-apply", "${env}-deploy"]]))
  project  = var.project_id
  role     = "roles/serviceusage.serviceUsageConsumer"
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
}
resource "google_project_iam_custom_role" "account_status" {
  role_id     = "ciproxyLaunchStatus"
  title       = "ciproxy launch status"
  permissions = ["iam.serviceAccounts.get"]
}
resource "google_service_account_iam_member" "status" {
  for_each           = { for pair in setproduct(local.retired, setunion(flatten([for env in local.environments : ["${env}-plan", "${env}-apply"]]), local.shared_accounts)) : "${pair[0]}:${pair[1]}" => pair }
  service_account_id = google_service_account.ci[each.value[0]].name
  role               = google_project_iam_custom_role.account_status.name
  member             = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
resource "google_tags_tag_key" "environment" {
  parent     = "projects/${data.google_project.current.number}"
  short_name = "ciproxy-environment"
}
resource "google_tags_tag_value" "environment" {
  for_each   = local.environments
  parent     = google_tags_tag_key.environment.id
  short_name = each.key
}
resource "google_project_iam_custom_role" "invalidate" {
  role_id     = "ciproxyInvalidateCdn"
  title       = "ciproxy own CDN invalidation"
  permissions = ["compute.urlMaps.invalidateCache"]
}
resource "google_project_iam_member" "invalidate" {
  for_each = local.environments
  project  = var.project_id
  role     = google_project_iam_custom_role.invalidate.name
  member   = "serviceAccount:${google_service_account.ci["${each.key}-deploy"].email}"
  condition {
    title      = "own-environment-edge"
    expression = "resource.matchTagId('${google_tags_tag_key.environment.id}', '${google_tags_tag_value.environment[each.key].id}')"
  }
}
resource "google_project_iam_custom_role" "tag_binding" {
  role_id     = "ciproxyEdgeTagBinding"
  title       = "ciproxy central edge tag binding"
  permissions = ["compute.urlMaps.createTagBinding", "compute.urlMaps.deleteTagBinding", "compute.urlMaps.listTagBindings", "compute.urlMaps.listEffectiveTags", "resourcemanager.tagValueBindings.create", "resourcemanager.tagValueBindings.delete", "resourcemanager.tagValues.get", "resourcemanager.tagKeys.get"]
}
resource "google_project_iam_member" "tag_binding" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.tag_binding.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
}
resource "google_tags_tag_value_iam_member" "tag_user" {
  for_each  = { for pair in setproduct(local.environments, local.shared_accounts) : "${pair[0]}:${pair[1]}" => pair if pair[1] == "shared_production" || pair[0] == "main-preview" }
  tag_value = google_tags_tag_value.environment[each.value[0]].id
  role      = "roles/resourcemanager.tagUser"
  member    = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
output "edge_tags" { value = { for env, tag in google_tags_tag_value.environment : env => tag.id } }

output "project_number" { value = data.google_project.current.number }

# The synchronous invalidate CLI polls a global operation. Operations do not
# inherit URL-map tags, so keep this metadata read in its own binding.
resource "google_project_iam_custom_role" "operation_reader" {
  role_id     = "ciproxyOperationReader"
  title       = "ciproxy CDN operation completion"
  permissions = ["compute.globalOperations.get"]
}
resource "google_project_iam_member" "operation_reader" {
  for_each = local.environments
  project  = var.project_id
  role     = google_project_iam_custom_role.operation_reader.name
  member   = "serviceAccount:${google_service_account.ci["${each.key}-deploy"].email}"
}

# Policy writes to an exact runtime account do not grant project IAM. Preview
# must never be able to preinstall an impersonation grant on production.
resource "google_project_iam_custom_role" "runtime_policy" {
  role_id     = "ciproxyRuntimePolicy"
  title       = "ciproxy exact runtime service account policy"
  permissions = ["iam.serviceAccounts.getIamPolicy", "iam.serviceAccounts.setIamPolicy"]
}
resource "google_service_account_iam_member" "runtime_policy" {
  for_each = {
    preview            = { environment = "main-preview", account = "shared_preview" }
    production_preview = { environment = "main-preview", account = "shared_production" }
    production         = { environment = "production", account = "shared_production" }
  }
  service_account_id = google_service_account.runtime[each.value.environment].name
  role               = google_project_iam_custom_role.runtime_policy.name
  member             = "serviceAccount:${google_service_account.ci[each.value.account].email}"
}
