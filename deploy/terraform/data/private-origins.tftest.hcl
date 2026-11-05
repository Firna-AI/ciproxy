mock_provider "google" {
  mock_data "google_dns_managed_zone" { defaults = { name_servers = ["ns1.example.invalid."] } }
}
variables {
  project_id  = "ciproxy"
  environment = "main-preview"
  domain      = "ciproxy.dev"
  shared = {
    network_id         = "projects/ciproxy/global/networks/ciproxy-shared"
    cluster_name       = "ciproxy-shared"
    runtime_repository = "europe-west1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime"
    allowed_zones      = ["europe-west1-b", "europe-west1-c", "europe-west1-d"]
  }
}
run "static_origins_enforce_pap_without_anonymous_publishers" {
  command = plan
  module { source = "../modules/foundation" }
  assert {
    condition = (
      alltrue([for bucket in google_storage_bucket.public : bucket.public_access_prevention == "enforced"])
      && google_storage_bucket.images.public_access_prevention == "enforced"
      && alltrue([for grant in google_storage_bucket_iam_member.publisher : !contains(["allUsers", "allAuthenticatedUsers"], grant.member)])
    )
    error_message = "Public website content must have private PAP-enforced origins and no anonymous publisher grants."
  }
}

run "production_static_origins_enforce_pap" {
  command = plan
  module { source = "../modules/foundation" }
  variables { environment = "production" }
  assert {
    condition     = alltrue([for bucket in google_storage_bucket.public : bucket.public_access_prevention == "enforced"]) && alltrue([for grant in google_storage_bucket_iam_member.publisher : !contains(["allUsers", "allAuthenticatedUsers"], grant.member)])
    error_message = "Production static origins must enforce PAP without anonymous publishers."
  }
}
