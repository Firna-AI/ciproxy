mock_provider "google" {}
override_data {
  target = data.terraform_remote_state.shared
  values = { outputs = { environments = { "main-preview" = { "edge" = { "project_id" = "ciproxy", "environment" = "main-preview", "prefix" = "ciproxy-main-preview", "allowed_zones" = ["europe-west1-b", "europe-west1-c"], "hosts" = { "site" = "ciproxy-main-preview.invalid", "dashboard" = "ciproxy-main-preview-app.invalid", "releases" = "ciproxy-main-preview-get.invalid" }, "dns_zones" = { "site" = "ciproxy-main-preview", "dashboard" = "ciproxy-main-preview", "releases" = "ciproxy-main-preview" }, "buckets" = { "site" = "ciproxy-main-preview-site-ciproxy", "dashboard" = "ciproxy-main-preview-dashboard-ciproxy" }, "images_bucket" = "ciproxy-main-preview-images-ciproxy", "notification_channels" = [] } }, "production" = { "edge" = { "project_id" = "ciproxy", "environment" = "production", "prefix" = "ciproxy-production", "allowed_zones" = ["europe-west1-b", "europe-west1-c"], "hosts" = { "site" = "ciproxy-production.invalid", "dashboard" = "ciproxy-production-app.invalid", "releases" = "ciproxy-production-get.invalid" }, "dns_zones" = { "site" = "ciproxy-production", "dashboard" = "ciproxy-production", "releases" = "ciproxy-production" }, "buckets" = { "site" = "ciproxy-production-site-ciproxy", "dashboard" = "ciproxy-production-dashboard-ciproxy" }, "images_bucket" = "ciproxy-production-images-ciproxy", "notification_channels" = [] } } } } }
}
override_data {
  target = data.terraform_remote_state.trust
  values = { outputs = { project_number = "123456789", edge_tags = { main-preview = "tagValues/1", production = "tagValues/2" } } }
}
override_data {
  target = data.terraform_remote_state.environment["main-preview"]
  values = { outputs = {
    runtime_flags       = { control_plane_enabled = true, enable_uptime_checks = true }
    notification_emails = ["main-preview@example.invalid"]
    allowed_zones       = ["europe-west1-b"]
  } }
}
override_resource {
  target          = google_monitoring_notification_channel.email["main-preview:main-preview@example.invalid"]
  override_during = plan
  values          = { name = "projects/ciproxy/notificationChannels/100" }
}
override_data {
  target = data.terraform_remote_state.environment["production"]
  values = { outputs = {
    runtime_flags       = { control_plane_enabled = false, enable_uptime_checks = false }
    notification_emails = ["production@example.invalid"]
    allowed_zones       = ["europe-west1-b"]
  } }
}
override_resource {
  target          = google_monitoring_notification_channel.email["production:production@example.invalid"]
  override_during = plan
  values          = { name = "projects/ciproxy/notificationChannels/200" }
}
run "central_edge_uses_each_completed_environment_snapshot" {
  command = plan
  assert {
    condition     = toset(module.edge["main-preview"].readiness.backend_groups.control_plane) == toset(["https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b/networkEndpointGroups/ciproxy-main-preview-control"]) && length(module.edge["production"].readiness.backend_groups.control_plane) == 0
    error_message = "Preview runtime and prepared zones must not come from production or newer shared settings."
  }
  assert {
    condition     = length(module.edge["main-preview"].readiness.uptime_hosts) == 5 && length(module.edge["production"].readiness.uptime_hosts) == 0
    error_message = "Preview uptime stays enabled while production uptime is disabled."
  }
  assert {
    condition     = google_monitoring_notification_channel.email["main-preview:main-preview@example.invalid"].labels.email_address == "main-preview@example.invalid" && google_monitoring_notification_channel.email["production:production@example.invalid"].labels.email_address == "production@example.invalid" && alltrue([for channels in module.edge["main-preview"].readiness.channels : toset(channels) == toset(["projects/ciproxy/notificationChannels/100"])])
    error_message = "Each environment keeps its own alert address and channel."
  }
}
