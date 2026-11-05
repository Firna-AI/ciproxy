# First bootstrap runs before GKE exists. The operator fills this map only
# after creating each KSA on the intended cluster. CI preserves trust state.
variable "runtime_service_account_uids" {
  type    = map(string)
  default = {}
}
output "runtime_service_account_uids" { value = var.runtime_service_account_uids }

resource "google_project_iam_custom_role" "cluster_update" {
  role_id     = "ciproxySharedClusterUpdate"
  title       = "ciproxy existing shared cluster updates"
  permissions = ["container.clusters.get", "container.clusters.update"]
}
resource "google_project_iam_member" "cluster_update" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.cluster_update.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
  condition {
    title      = "intended-shared-cluster"
    expression = "resource.type == 'container.googleapis.com/Cluster' && resource.name == 'projects/${var.project_id}/locations/europe-west1/clusters/ciproxy-shared'"
  }
}
# Production is protected main. Prelaunch cannot create or replace a cluster.
resource "google_project_iam_custom_role" "cluster_lifecycle" {
  role_id     = "ciproxyProductionClusterLifecycle"
  title       = "ciproxy protected production cluster lifecycle"
  permissions = ["container.clusters.create", "container.clusters.delete"]
}
resource "google_project_iam_member" "cluster_lifecycle" {
  project = var.project_id
  role    = google_project_iam_custom_role.cluster_lifecycle.name
  member  = "serviceAccount:${google_service_account.ci["shared_production"].email}"
}
