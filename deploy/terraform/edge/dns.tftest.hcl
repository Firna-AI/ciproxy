mock_provider "google" {}
variables {
  project_number  = "123456789"
  environment_tag = "tagValues/1"
  zone_map        = { europe-west1-b = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b" }
}
override_resource {
  target          = google_compute_global_address.public
  override_during = plan
  values          = { address = "203.0.113.7" }
}
run "production_edge_manages_only_owned_dns_records" {
  command = plan
  module { source = "../modules/edge" }
  variables {
    configuration = {
      project_id    = "ciproxy"
      environment   = "production"
      prefix        = "ciproxy-production"
      allowed_zones = ["europe-west1-b"]
      hosts = {
        site      = "ciproxy.dev"
        dashboard = "app.ciproxy.dev"
        releases  = "get.ciproxy.dev"
      }
      dns_zones = {
        dashboard = "ciproxy-production-dashboard"
        releases  = "ciproxy-production-releases"
      }
      buckets = {
        site      = "ciproxy-production-site-ciproxy"
        dashboard = "ciproxy-production-dashboard-ciproxy"
      }
      images_bucket         = "ciproxy-production-images-ciproxy"
      notification_channels = []
    }
  }
  assert {
    condition = (
      toset(keys(google_dns_record_set.public)) == toset(["dashboard", "releases"])
      && google_dns_record_set.public["dashboard"].name == "app.ciproxy.dev."
      && google_dns_record_set.public["dashboard"].managed_zone == "ciproxy-production-dashboard"
      && google_dns_record_set.public["releases"].name == "get.ciproxy.dev."
      && google_dns_record_set.public["releases"].managed_zone == "ciproxy-production-releases"
      && alltrue([for record in google_dns_record_set.public : record.type == "A" && toset(record.rrdatas) == toset(["203.0.113.7"])])
    )
    error_message = "Edge A records must stay within the environment's delegated zones. Production manages no apex."
  }
  assert {
    condition = (
      toset(google_compute_managed_ssl_certificate.public.managed[0].domains) == toset(["ciproxy.dev", "app.ciproxy.dev", "get.ciproxy.dev"])
      && anytrue([for rule in google_compute_url_map.https.host_rule : contains(rule.hosts, "ciproxy.dev")])
      && google_compute_backend_bucket.public["site"].bucket_name == "ciproxy-production-site-ciproxy"
    )
    error_message = "The apex/site host remains on the certificate and HTTPS static backend."
  }
}
run "main_preview_edge_manages_only_owned_dns_records" {
  command = plan
  module { source = "../modules/edge" }
  variables {
    configuration = {
      project_id    = "ciproxy"
      environment   = "main-preview"
      prefix        = "ciproxy-main-preview"
      allowed_zones = ["europe-west1-b"]
      hosts = {
        site      = "br-main.preview.ciproxy.dev"
        dashboard = "br-main.preview.app.ciproxy.dev"
        releases  = "br-main.preview.get.ciproxy.dev"
      }
      dns_zones = {
        dashboard = "ciproxy-main-preview-dashboard"
        releases  = "ciproxy-main-preview-releases"
        site      = "ciproxy-main-preview"
      }
      buckets = {
        site      = "ciproxy-main-preview-site-ciproxy"
        dashboard = "ciproxy-main-preview-dashboard-ciproxy"
      }
      images_bucket         = "ciproxy-main-preview-images-ciproxy"
      notification_channels = []
    }
  }
  assert {
    condition = (
      toset(keys(google_dns_record_set.public)) == toset(["dashboard", "releases", "site"])
      && google_dns_record_set.public["dashboard"].name == "br-main.preview.app.ciproxy.dev."
      && google_dns_record_set.public["dashboard"].managed_zone == "ciproxy-main-preview-dashboard"
      && google_dns_record_set.public["releases"].name == "br-main.preview.get.ciproxy.dev."
      && google_dns_record_set.public["releases"].managed_zone == "ciproxy-main-preview-releases"
      && alltrue([for record in google_dns_record_set.public : record.type == "A" && toset(record.rrdatas) == toset(["203.0.113.7"])])
    )
    error_message = "Edge A records must stay within the environment's delegated zones. Production manages no apex."
  }
  assert {
    condition = (
      toset(google_compute_managed_ssl_certificate.public.managed[0].domains) == toset(["br-main.preview.ciproxy.dev", "br-main.preview.app.ciproxy.dev", "br-main.preview.get.ciproxy.dev"])
      && anytrue([for rule in google_compute_url_map.https.host_rule : contains(rule.hosts, "br-main.preview.ciproxy.dev")])
      && google_compute_backend_bucket.public["site"].bucket_name == "ciproxy-main-preview-site-ciproxy"
    )
    error_message = "The apex/site host remains on the certificate and HTTPS static backend."
  }
}
