mock_provider "google" {
  override_during = plan
  mock_data "google_dns_managed_zone" { defaults = { name_servers = ["ns1.example.invalid."] } }
  mock_resource "google_kms_crypto_key" {
    defaults = { id = "projects/ciproxy/locations/us-central1/keyRings/ciproxy-main-preview/cryptoKeys/boat-account-keys" }
  }
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
run "preview_deploy_gets_its_optional_snapshot_parent" {
  command = plan
  module { source = "../modules/foundation" }
  override_resource {
    target          = google_secret_manager_secret.boat_shared_snapshot
    override_during = plan
    values          = { id = "projects/ciproxy/secrets/ciproxy-main-preview-boat-shared-snapshot" }
  }
  assert {
    condition     = try(output.deployment.boat_shared_snapshot_secret, "") == "ciproxy-main-preview-boat-shared-snapshot"
    error_message = "Preview deploy needs its own optional Boat snapshot parent."
  }
  assert {
    condition = (
      google_secret_manager_secret_iam_member.boat_snapshot_sync.secret_id == google_secret_manager_secret.boat_shared_snapshot.id
      && google_secret_manager_secret_iam_member.boat_snapshot_sync.role == google_secret_manager_secret_iam_member.release_sync.role
      && google_secret_manager_secret_iam_member.boat_snapshot_sync.member == "serviceAccount:ciproxy-main-preview-deploy@ciproxy.iam.gserviceaccount.com"
      && !contains(keys(output.deployment.secrets), "boat-shared-snapshot")
    )
    error_message = "Only the preview deploy identity syncs the optional value, with the release-token access role."
  }
  assert {
    condition = (
      output.deployment.kms_key == "projects/ciproxy/locations/us-central1/keyRings/ciproxy-main-preview/cryptoKeys/boat-account-keys"
      && google_kms_crypto_key_iam_member.control_plane.role == "roles/cloudkms.cryptoKeyEncrypterDecrypter"
      && google_kms_crypto_key_iam_member.control_plane.member == "serviceAccount:ciproxy-main-preview-control@ciproxy.iam.gserviceaccount.com"
    )
    error_message = "Boat needs the full key path and runtime encrypt/decrypt access."
  }
}
run "production_deploy_gets_its_own_optional_snapshot_parent" {
  command = plan
  module { source = "../modules/foundation" }
  variables { environment = "production" }
  override_resource {
    target          = google_kms_crypto_key.boat
    override_during = plan
    values          = { id = "projects/ciproxy/locations/us-central1/keyRings/ciproxy-production/cryptoKeys/boat-account-keys" }
  }
  override_resource {
    target          = google_secret_manager_secret.boat_shared_snapshot
    override_during = plan
    values          = { id = "projects/ciproxy/secrets/ciproxy-production-boat-shared-snapshot" }
  }
  assert {
    condition     = try(output.deployment.boat_shared_snapshot_secret, "") == "ciproxy-production-boat-shared-snapshot"
    error_message = "Production deploy needs its own optional Boat snapshot parent."
  }
  assert {
    condition = (
      google_secret_manager_secret_iam_member.boat_snapshot_sync.secret_id == google_secret_manager_secret.boat_shared_snapshot.id
      && google_secret_manager_secret_iam_member.boat_snapshot_sync.role == "roles/secretmanager.secretAccessor"
      && google_secret_manager_secret_iam_member.boat_snapshot_sync.member == "serviceAccount:ciproxy-production-deploy@ciproxy.iam.gserviceaccount.com"
      && output.deployment.kms_key == "projects/ciproxy/locations/us-central1/keyRings/ciproxy-production/cryptoKeys/boat-account-keys"
      && google_kms_crypto_key_iam_member.control_plane.member == "serviceAccount:ciproxy-production-control@ciproxy.iam.gserviceaccount.com"
    )
    error_message = "Production keeps its snapshot reader and Boat key in production."
  }
}
