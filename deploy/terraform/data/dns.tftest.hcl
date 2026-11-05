mock_provider "google" {}
variables {
  project_id  = "ciproxy"
  environment = "production"
  domain      = "ciproxy.dev"
  shared = {
    network_id         = "projects/ciproxy/global/networks/ciproxy-shared"
    cluster_name       = "ciproxy-shared"
    runtime_repository = "us-central1-cocker.pkg.dev/ciproxy/ciproxy-shared-runtime"
    allowed_zones      = ["us-central1-a", "us-central1-b", "us-central1-c"]
  }
}
run "production_owns_only_delegated_app_and_get_zones" {
  command = plan
  module { source = "../modules/foundation" }
  override_data {
    target = data.google_dns_managed_zone.preview["dashboard"]
    values = { name_servers = ["ns-cloud-a1.googledomains.com.", "ns-cloud-a2.googledomains.com."] }
  }
  override_data {
    target = data.google_dns_managed_zone.preview["releases"]
    values = { name_servers = ["ns-cloud-b1.googledomains.com.", "ns-cloud-b2.googledomains.com."] }
  }
  assert {
    condition = (
      toset(keys(google_dns_managed_zone.public)) == toset(["dashboard", "releases"])
      && google_dns_managed_zone.public["dashboard"].name == "ciproxy-production-dashboard"
      && google_dns_managed_zone.public["dashboard"].dns_name == "app.ciproxy.dev."
      && google_dns_managed_zone.public["releases"].name == "ciproxy-production-releases"
      && google_dns_managed_zone.public["releases"].dns_name == "get.ciproxy.dev."
    )
    error_message = "Production owns app/get zones only. Cloudflare keeps the apex and Workspace mail."
  }
  assert {
    condition = (
      toset(keys(google_dns_record_set.preview_delegation)) == toset(["dashboard", "releases"])
      && data.google_dns_managed_zone.preview["dashboard"].name == "ciproxy-main-preview-dashboard"
      && data.google_dns_managed_zone.preview["releases"].name == "ciproxy-main-preview-releases"
      && google_dns_record_set.preview_delegation["dashboard"].name == "preview.app.ciproxy.dev."
      && google_dns_record_set.preview_delegation["dashboard"].managed_zone == "ciproxy-production-dashboard"
      && google_dns_record_set.preview_delegation["dashboard"].type == "NS"
      && toset(google_dns_record_set.preview_delegation["dashboard"].rrdatas) == toset(["ns-cloud-a1.googledomains.com.", "ns-cloud-a2.googledomains.com."])
      && google_dns_record_set.preview_delegation["releases"].name == "preview.get.ciproxy.dev."
      && google_dns_record_set.preview_delegation["releases"].managed_zone == "ciproxy-production-releases"
      && google_dns_record_set.preview_delegation["releases"].type == "NS"
      && toset(google_dns_record_set.preview_delegation["releases"].rrdatas) == toset(["ns-cloud-b1.googledomains.com.", "ns-cloud-b2.googledomains.com."])
    )
    error_message = "Production app/get parents must delegate preview children using canonical Google zone reads."
  }

}

run "preview_keeps_three_original_zones_without_parent_delegations" {
  command = plan
  module { source = "../modules/foundation" }
  variables { environment = "main-preview" }
  assert {
    condition = (
      toset(keys(google_dns_managed_zone.public)) == toset(["site", "dashboard", "releases"])
      && google_dns_managed_zone.public["site"].dns_name == "preview.ciproxy.dev."
      && google_dns_managed_zone.public["dashboard"].dns_name == "preview.app.ciproxy.dev."
      && google_dns_managed_zone.public["releases"].dns_name == "preview.get.ciproxy.dev."
      && length(google_dns_record_set.preview_delegation) == 0
    )
    error_message = "Preview retains its three zones and creates no production parent delegations."
  }
}
