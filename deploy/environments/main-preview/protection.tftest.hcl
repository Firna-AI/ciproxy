mock_provider "google" {}

run "database_and_secret_protection" {
  command = plan
  module {
    source = "../../terraform/modules/foundation"
  }
  variables {
    project_id  = "ciproxy"
    environment = "main-preview"
    domain      = "ciproxy.dev"
    shared = {
      network_id         = "projects/ciproxy/global/networks/ciproxy-shared"
      allowed_zones      = ["europe-west1-b", "europe-west1-c", "europe-west1-d"]
      cluster_name       = "ciproxy-shared"
      runtime_repository = "europe-west1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime"
    }
  }
  assert {
    condition = (
      google_sql_database_instance.main.deletion_protection
      && google_sql_database_instance.main.settings[0].deletion_protection_enabled
      && google_secret_manager_secret.release.deletion_protection
      && alltrue([for secret in google_secret_manager_secret.control_plane : secret.deletion_protection])
    )
    error_message = "Cloud SQL needs API and provider protection. Each secret needs provider protection."
  }
}
