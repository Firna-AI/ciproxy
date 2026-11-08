variable "control_plane_registry_ready" {
  type    = bool
  default = false
}
output "control_plane_registry_ready" { value = var.control_plane_registry_ready }

locals {
  image_permissions = {
    image_lookup  = ["artifactregistry.repositories.get", "artifactregistry.repositories.downloadArtifacts"]
    image_publish = ["artifactregistry.repositories.get", "artifactregistry.repositories.downloadArtifacts", "artifactregistry.repositories.uploadArtifacts"]
  }
}
resource "google_project_iam_custom_role" "control_plane_image" {
  for_each    = local.image_permissions
  role_id     = each.key == "image_lookup" ? "ciproxyImageLookup" : "ciproxyImagePublish"
  title       = "ciproxy ${replace(each.key, "_", " ")}"
  permissions = each.value
}
# Bootstrap precedes shared core at first setup. The operator enables these
# bindings once the shared Artifact Registry repository exists.
resource "google_artifact_registry_repository_iam_member" "control_plane_image" {
  for_each   = var.control_plane_registry_ready ? local.image_permissions : {}
  project    = var.project_id
  location   = "us-central1"
  repository = "ciproxy-shared-runtime"
  role       = google_project_iam_custom_role.control_plane_image[each.key].name
  member     = "serviceAccount:${google_service_account.ci[each.key].email}"
}

resource "google_project_iam_custom_role" "image_policy" {
  role_id     = "ciproxyImageRepositoryPolicy"
  title       = "ciproxy exact image repository policy maintenance"
  permissions = ["artifactregistry.repositories.getIamPolicy", "artifactregistry.repositories.setIamPolicy"]
}
resource "google_artifact_registry_repository_iam_member" "image_policy" {
  for_each   = var.control_plane_registry_ready ? toset(["trust_preview", "trust_production"]) : toset([])
  project    = var.project_id
  location   = "us-central1"
  repository = "ciproxy-shared-runtime"
  role       = google_project_iam_custom_role.image_policy.name
  member     = "serviceAccount:${google_service_account.ci[each.key].email}"
}
