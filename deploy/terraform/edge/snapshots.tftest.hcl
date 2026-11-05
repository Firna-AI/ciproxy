mock_provider "google" {}
variables {
  environment = "main-preview"
  workload = {
    allowed_zones         = ["prepared-west"]
    control_plane_enabled = true
    enable_uptime_checks  = true
    notification_emails   = ["preview@example.invalid"]
    source_sha            = "0000000000000000000000000000000000000000"
  }
}
override_module {
  target  = module.zones
  outputs = { catalog = { prepared-west = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b" } }
}
override_data {
  target = data.terraform_remote_state.trust
  values = { outputs = { project_number = "123456789", edge_tags = { main-preview = "tagValues/1", production = "tagValues/2" } } }
}
run "main_preview_edge_uses_evidence_catalog_and_permanent_tag_id" {
  command = plan
  variables { environment = "main-preview" }
  override_data {
    target = data.terraform_remote_state.infrastructure
    values = { outputs = { edge = { "project_id" = "ciproxy", "environment" = "main-preview", "prefix" = "ciproxy-main-preview", "allowed_zones" = ["europe-west1-d"], "hosts" = { "site" = "ciproxy-main-preview.invalid", "dashboard" = "ciproxy-main-preview-app.invalid", "releases" = "ciproxy-main-preview-get.invalid" }, "dns_zones" = { "site" = "ciproxy-main-preview", "dashboard" = "ciproxy-main-preview", "releases" = "ciproxy-main-preview" }, "buckets" = { "site" = "ciproxy-main-preview-site-ciproxy", "dashboard" = "ciproxy-main-preview-dashboard-ciproxy" }, "images_bucket" = "ciproxy-main-preview-images-ciproxy", "notification_channels" = [] } } }
  }
  override_resource {
    target          = module.edge.google_compute_url_map.https
    override_during = plan
    values          = { map_id = 900000001 }
  }
  assert {
    condition     = module.edge.tag_parent == "//compute.googleapis.com/projects/123456789/global/urlMaps/900000001" && module.edge.url_map == "ciproxy-main-preview"
    error_message = "Tag binding must use the permanent map ID. Invalidation must keep its name."
  }
  assert {
    condition     = toset(module.edge.readiness.backend_groups.control_plane) == toset(["https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b/networkEndpointGroups/ciproxy-main-preview-control"])
    error_message = "Completed evidence selects a trusted catalog URL, including a previous prepared zone."
  }
  assert {
    condition     = length(module.edge.readiness.uptime_hosts) == 5 && google_monitoring_notification_channel.email["preview@example.invalid"].labels.email_address == "preview@example.invalid"
    error_message = "The selected evidence keeps its own runtime, uptime, and alert inputs."
  }
}
run "production_edge_uses_evidence_catalog_and_permanent_tag_id" {
  command = plan
  variables { environment = "production" }
  override_data {
    target = data.terraform_remote_state.infrastructure
    values = { outputs = { edge = { "project_id" = "ciproxy", "environment" = "production", "prefix" = "ciproxy-production", "allowed_zones" = ["europe-west1-d"], "hosts" = { "site" = "ciproxy-production.invalid", "dashboard" = "ciproxy-production-app.invalid", "releases" = "ciproxy-production-get.invalid" }, "dns_zones" = { "site" = "ciproxy-production", "dashboard" = "ciproxy-production", "releases" = "ciproxy-production" }, "buckets" = { "site" = "ciproxy-production-site-ciproxy", "dashboard" = "ciproxy-production-dashboard-ciproxy" }, "images_bucket" = "ciproxy-production-images-ciproxy", "notification_channels" = [] } } }
  }
  override_resource {
    target          = module.edge.google_compute_url_map.https
    override_during = plan
    values          = { map_id = 900000002 }
  }
  assert {
    condition     = module.edge.tag_parent == "//compute.googleapis.com/projects/123456789/global/urlMaps/900000002" && module.edge.url_map == "ciproxy-production"
    error_message = "Tag binding must use the permanent map ID. Invalidation must keep its name."
  }
  assert {
    condition     = toset(module.edge.readiness.backend_groups.control_plane) == toset(["https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b/networkEndpointGroups/ciproxy-production-control"])
    error_message = "Completed evidence selects a trusted catalog URL, including a previous prepared zone."
  }
  assert {
    condition     = length(module.edge.readiness.uptime_hosts) == 5 && google_monitoring_notification_channel.email["preview@example.invalid"].labels.email_address == "preview@example.invalid"
    error_message = "The selected evidence keeps its own runtime, uptime, and alert inputs."
  }
}
