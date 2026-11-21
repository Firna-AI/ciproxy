mock_provider "google" {
  mock_data "google_project" { defaults = { number = "123456789" } }
}
override_resource {
  target          = google_project_iam_custom_role.bucket_policy
  override_during = plan
  values          = { name = "projects/ciproxy/roles/ciproxyBucketPolicy" }
}
override_resource {
  target          = google_service_account.ci["shared_preview"]
  override_during = plan
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
  }
}
override_resource {
  target          = google_service_account.ci["shared_production"]
  override_during = plan
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
  }
}
variables {
  project_id         = "ciproxy"
  production_enabled = false
}
run "bucket_policy_setters_cover_only_phase_owned_static_origins" {
  command = plan
  assert {
    condition = (
      google_project_iam_member.static_bucket_policy["shared_preview"].member == "serviceAccount:ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
      && google_project_iam_member.static_bucket_policy["shared_production"].member == "serviceAccount:ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
      && toset(google_project_iam_custom_role.bucket_metadata.permissions) == toset(["storage.buckets.get", "storage.buckets.update", "storage.buckets.delete"])
      && toset(google_project_iam_custom_role.bucket_policy.permissions) == toset(["storage.buckets.getIamPolicy", "storage.buckets.setIamPolicy"])
      && google_project_iam_member.static_bucket_policy["shared_preview"].condition[0].expression == "resource.name == 'projects/_/buckets/ciproxy-main-preview-site-ciproxy' || resource.name == 'projects/_/buckets/ciproxy-main-preview-dashboard-ciproxy'"
      && google_project_iam_member.static_bucket_policy["shared_production"].condition[0].expression == "resource.name == 'projects/_/buckets/ciproxy-main-preview-site-ciproxy' || resource.name == 'projects/_/buckets/ciproxy-main-preview-dashboard-ciproxy' || resource.name == 'projects/_/buckets/ciproxy-production-site-ciproxy' || resource.name == 'projects/_/buckets/ciproxy-production-dashboard-ciproxy'"
    )
    error_message = "Shared policy get/set must name preview2 or production4 static buckets, never image/state/trust buckets."
  }
  assert {
    condition = alltrue([for key, grant in google_storage_bucket_iam_member.trust_policy :
      grant.role == google_project_iam_custom_role.bucket_policy.name
      && contains(["trust_preview", "trust_production"], split(":", key)[1])
    ])
    error_message = "State-bucket IAM maintenance belongs only to main-only trusted bootstrap."
  }
}
