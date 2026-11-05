mock_provider "google" {
  mock_data "google_dns_managed_zone" { defaults = { name_servers = ["ns1.example.invalid."] } }
}
override_data {
  target = data.terraform_remote_state.shared
  values = { outputs = { platform = {
    network_id         = "projects/ciproxy/global/networks/ciproxy-shared"
    cluster_name       = "ciproxy-shared"
    runtime_repository = "us-central1-cocker.pkg.dev/ciproxy/ciproxy-shared-runtime"
    allowed_zones      = ["us-central1-a", "us-central1-b", "us-central1-c"]
  } } }
}
variables { environment = "main-preview" }
run "preview_data_has_environment_names" {
  command = plan
  assert {
    condition     = module.environment.deployment.namespace == "ciproxy-main-preview" && module.environment.deployment.sql_instance == "ciproxy-main-preview"
    error_message = "Preview data keeps preview SQL and namespace names."
  }
}
run "production_data_does_not_adopt_preview_resource_ids" {
  command = plan
  variables { environment = "production" }
  assert {
    condition     = module.environment.deployment.namespace == "ciproxy-production" && module.environment.deployment.sql_instance == "ciproxy-production"
    error_message = "Production creates its own named data resources in protected state."
  }
}
