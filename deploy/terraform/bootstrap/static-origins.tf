resource "google_project_iam_custom_role" "bucket_policy" {
  role_id     = "ciproxyBucketPolicy"
  title       = "ciproxy exact bucket IAM policies"
  permissions = ["storage.buckets.getIamPolicy", "storage.buckets.setIamPolicy"]
}
resource "google_project_iam_member" "static_bucket_policy" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.bucket_policy.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
  condition {
    title = "static-origin-policies-only"
    expression = join(" || ", [for pair in setproduct(local.account_environments[each.key], ["site", "dashboard"]) :
      "resource.name == 'projects/_/buckets/ciproxy-${pair[0]}-${pair[1]}-${var.project_id}'"
    ])
  }
}
# Trusted bootstrap maintains state IAM independently. Shared apply never gets
# this resource binding, and its project policy condition excludes these names.
resource "google_storage_bucket_iam_member" "trust_policy" {
  for_each = { for pair in setproduct(local.state_scopes, ["trust_preview", "trust_production"]) : "${pair[0]}:${pair[1]}" => pair }
  bucket   = "ciproxy-${each.value[0]}-tfstate-${var.project_id}"
  role     = google_project_iam_custom_role.bucket_policy.name
  member   = "serviceAccount:${google_service_account.ci[each.value[1]].email}"
}
