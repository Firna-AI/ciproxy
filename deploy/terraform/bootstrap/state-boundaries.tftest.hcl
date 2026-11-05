mock_provider "google" {
  mock_data "google_project" { defaults = { number = "123456789" } }
}
variables {
  project_id         = "ciproxy"
  production_enabled = false
}
run "unreviewed_writers_cannot_poison_managed_production_state" {
  command = plan
  assert {
    condition = (
      !contains(keys(google_storage_bucket_iam_member.writer), "production:shared_preview")
      && !contains(keys(google_storage_bucket_iam_member.reader), "production:shared_preview")
      && !contains(keys(google_storage_bucket_iam_member.plan_lock), "production:shared_plan")
      && google_storage_bucket_iam_member.writer["main-preview:main-preview-apply"].condition[0].expression == "resource.name.startsWith('projects/_/buckets/ciproxy-main-preview-tfstate-ciproxy/objects/main-preview/foundation/')"
      && google_storage_bucket_iam_member.writer["production:production-apply"].condition[0].expression == "resource.name.startsWith('projects/_/buckets/ciproxy-production-tfstate-ciproxy/objects/production/foundation/')"
      && google_storage_bucket_iam_member.writer["production:shared_production"].condition[0].expression == "resource.name.startsWith('projects/_/buckets/ciproxy-production-tfstate-ciproxy/objects/production/data/') || resource.name.startsWith('projects/_/buckets/ciproxy-production-tfstate-ciproxy/objects/production/edge/')"
    )
    error_message = "Prelaunch has no production managed state/lock grant. Snapshot writers own only foundation records."
  }
  assert {
    condition = (
      !strcontains(google_project_iam_member.shared_secret["shared_preview"].condition[0].expression, "ciproxy-production-")
      && !strcontains(google_project_iam_member.shared_kms["shared_preview"].condition[0].expression, "ciproxy-production")
      && !strcontains(google_project_iam_member.bucket_metadata["shared_preview"].condition[0].expression, "ciproxy-production-")
      && !contains(keys(google_tags_tag_value_iam_member.tag_user), "production:shared_preview")
    )
    error_message = "Prelaunch secret, key, bucket, and tag policy grants must remain preview-only."
  }
}
run "shared_registry_and_platform_cannot_retain_prelaunch_iam_grants" {
  command = plan
  assert {
    condition = (
      toset([for key, grant in google_project_iam_member.shared : grant.role if startswith(key, "shared_preview:")]) == toset(["roles/cloudsql.admin", "roles/servicenetworking.networksAdmin", "roles/serviceusage.serviceUsageAdmin"])
      && alltrue([for permission in setunion(google_project_iam_custom_role.prelaunch.permissions, google_project_iam_custom_role.registry.permissions, google_project_iam_custom_role.shared_api.permissions) : !endswith(permission, ".setIamPolicy")])
      && toset(google_project_iam_custom_role.registry.permissions) == toset(["artifactregistry.repositories.create", "artifactregistry.repositories.get", "artifactregistry.repositories.update", "artifactregistry.repositories.delete"])
      && google_project_iam_member.registry_nodes.role == "roles/artifactregistry.reader"
    )
    error_message = "Registry/node grants belong to trusted bootstrap. Prelaunch shared APIs have no persistent IAM setters."
  }
}
