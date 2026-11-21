resource "google_project_iam_custom_role" "registry" {
  role_id     = "ciproxyRegistryLifecycle"
  title       = "ciproxy registry lifecycle without policy writes"
  permissions = ["artifactregistry.repositories.create", "artifactregistry.repositories.get", "artifactregistry.repositories.update", "artifactregistry.repositories.delete"]
}
resource "google_project_iam_member" "registry" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.registry.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
}
resource "google_project_iam_member" "registry_nodes" {
  project = var.project_id
  role    = "roles/artifactregistry.reader"
  member  = "serviceAccount:${google_service_account.nodes.email}"
}
