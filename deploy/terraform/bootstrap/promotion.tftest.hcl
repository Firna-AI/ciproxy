mock_provider "google" {}

variables {
  project_id  = "internal-498318"
  environment = "main-preview"
}

override_resource {
  target          = google_iam_workload_identity_pool.github
  override_during = plan
  values = {
    name = "projects/123/locations/global/workloadIdentityPools/ciproxy-main-preview"
  }
}

run "preview_identity_boundaries" {
  command = plan

  assert {
    condition = google_iam_workload_identity_pool_provider.promotion["apply"].attribute_condition == join(" && ", [
      "assertion.repository_id == '1442045837'",
      "assertion.repository_owner_id == '307519338'",
      "assertion.environment == 'main-preview'",
      "assertion.event_name == 'push'",
      "assertion.workflow_ref == 'Firna-AI/ciproxy/.github/workflows/deploy-main-preview.yml@' + assertion.ref",
      "assertion.ref in ['refs/heads/calummoore/ci-proxy-load-balancer', 'refs/heads/main']",
      "assertion.job_workflow_ref == 'Firna-AI/ciproxy/.github/workflows/deploy-apply.yml@' + assertion.ref",
    ])
    error_message = "Only the trusted caller and reusable workflow can use apply credentials."
  }

  assert {
    condition = alltrue([for access in ["plan", "apply"] :
      google_service_account_iam_member.promotion[access].member == "principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ciproxy-main-preview/attribute.access/${access}"
      && google_iam_workload_identity_pool_provider.promotion[access].attribute_mapping["attribute.access"] == "'${access}'"
    ])
    error_message = "Plan, workload, and apply credentials must use distinct access attributes."
  }

  assert {
    condition = (
      google_service_account_iam_member.federation.member == "principalSet://iam.googleapis.com/projects/123/locations/global/workloadIdentityPools/ciproxy-main-preview/attribute.access/deploy"
      && google_iam_workload_identity_pool_provider.github.attribute_mapping["attribute.access"] == "'deploy'"
      && toset(google_project_iam_custom_role.plan_lock.permissions) == toset(["storage.objects.create", "storage.objects.delete"])
      && google_storage_bucket_iam_member.plan_lock.condition[0].expression == join(" || ", [
        "resource.name == 'projects/_/buckets/ciproxy-main-preview-tfstate-internal-498318/objects/main-preview/seed/default.tflock'",
        "resource.name == 'projects/_/buckets/ciproxy-main-preview-tfstate-internal-498318/objects/main-preview/bootstrap/default.tflock'",
        "resource.name == 'projects/_/buckets/ciproxy-main-preview-tfstate-internal-498318/objects/main-preview/foundation/default.tflock'",
        "resource.name == 'projects/_/buckets/ciproxy-main-preview-tfstate-internal-498318/objects/main-preview/edge/default.tflock'",
      ])
    )
    error_message = "The plan account can write only four exact Terraform lock objects."
  }

  assert {
    condition = (
      toset([for grant in google_project_iam_member.deploy : grant.role]) == toset([
        "roles/compute.loadBalancerAdmin", "roles/compute.networkViewer",
        "roles/container.developer", "roles/dns.admin", "roles/monitoring.editor",
        "roles/serviceusage.serviceUsageConsumer",
      ])
      && alltrue([for grant in google_project_iam_member.plan : !can(regex("Admin|admin|editor|owner", grant.role))])
    )
    error_message = "Workload roles stay unchanged and plan roles remain read-only."
  }
}

run "production_identity_requires_main_workflow_run" {
  command = plan
  variables {
    environment = "production"
    project_id  = "ciproxy"
  }
  assert {
    condition = (
      strcontains(google_iam_workload_identity_pool_provider.promotion["apply"].attribute_condition, "assertion.ref == 'refs/heads/main'")
      && strcontains(google_iam_workload_identity_pool_provider.promotion["apply"].attribute_condition, "assertion.event_name == 'workflow_run'")
      && strcontains(google_iam_workload_identity_pool_provider.promotion["apply"].attribute_condition, "deploy-production.yml@' + assertion.ref")
      && !strcontains(google_iam_workload_identity_pool_provider.promotion["apply"].attribute_condition, "refs/tags/")
    )
    error_message = "Production trusts the main workflow_run caller and no deploy tag."
  }
}
