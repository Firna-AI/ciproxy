mock_provider "google" {
  override_during = plan
  mock_data "google_dns_managed_zone" { defaults = { name_servers = ["ns1.example.invalid."] } }
}
variables {
  project_id  = "ciproxy"
  environment = "main-preview"
  domain      = "ciproxy.dev"
  shared = {
    network_id         = "projects/ciproxy/global/networks/ciproxy-shared"
    cluster_name       = "ciproxy-shared"
    runtime_repository = "us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime"
    allowed_zones      = ["us-central1-a", "us-central1-b", "us-central1-c"]
  }
}
run "preview_deploy_gets_its_optional_r2_images_parent" {
  command = plan
  module { source = "../modules/foundation" }
  override_resource {
    target          = google_secret_manager_secret.r2_images
    override_during = plan
    values          = { id = "projects/ciproxy/secrets/ciproxy-main-preview-r2-images" }
  }
  assert {
    condition     = try(output.deployment.r2_images_secret, "") == "ciproxy-main-preview-r2-images"
    error_message = "Preview deploy needs its own optional R2 images key parent."
  }
  assert {
    condition = (
      google_secret_manager_secret_iam_member.r2_images_sync.secret_id == google_secret_manager_secret.r2_images.id
      && google_secret_manager_secret_iam_member.r2_images_sync.role == google_secret_manager_secret_iam_member.release_sync.role
      && google_secret_manager_secret_iam_member.r2_images_sync.member == "serviceAccount:ciproxy-main-preview-deploy@ciproxy.iam.gserviceaccount.com"
      && !contains(keys(output.deployment.secrets), "r2-images")
    )
    error_message = "Only the preview deploy identity copies the optional key, and the pod does not mount it."
  }
}
run "production_deploy_gets_its_own_optional_r2_images_parent" {
  command = plan
  module { source = "../modules/foundation" }
  variables { environment = "production" }
  override_resource {
    target          = google_secret_manager_secret.r2_images
    override_during = plan
    values          = { id = "projects/ciproxy/secrets/ciproxy-production-r2-images" }
  }
  assert {
    condition = (
      try(output.deployment.r2_images_secret, "") == "ciproxy-production-r2-images"
      && google_secret_manager_secret_iam_member.r2_images_sync.secret_id == google_secret_manager_secret.r2_images.id
      && google_secret_manager_secret_iam_member.r2_images_sync.member == "serviceAccount:ciproxy-production-deploy@ciproxy.iam.gserviceaccount.com"
    )
    error_message = "Production keeps its own R2 images key parent and reader."
  }
}
