mock_provider "google" {
  mock_data "google_project" { defaults = { number = "123456789" } }
  mock_resource "google_service_account" {
    defaults = {
      name  = "projects/ciproxy/serviceAccounts/mock-account@ciproxy.iam.gserviceaccount.com"
      email = "mock-account@ciproxy.iam.gserviceaccount.com"
    }
  }
}
variables {
  project_id         = "ciproxy"
  production_enabled = false
}
override_resource {
  target = google_service_account.ci["image_lookup"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-image-lookup@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-image-lookup@ciproxy.iam.gserviceaccount.com"
  }
}
override_resource {
  target = google_service_account.ci["image_publish"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-image-publish@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-image-publish@ciproxy.iam.gserviceaccount.com"
  }
}
override_resource {
  target = google_service_account.ci["trust_preview"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com"
  }
}
override_resource {
  target = google_service_account.ci["trust_production"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-trust-prod-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-trust-prod-apply@ciproxy.iam.gserviceaccount.com"
  }
}
run "first_bootstrap_needs_no_existing_registry" {
  command = plan
  assert {
    condition     = length(google_artifact_registry_repository_iam_member.control_plane_image) == 0 && length(google_artifact_registry_repository_iam_member.image_policy) == 0 && !output.control_plane_registry_ready
    error_message = "Initial trust setup must work before the shared repository exists."
  }
}
run "publish_is_repo_scoped_and_trusted_preview_only" {
  command = apply
  variables { control_plane_registry_ready = true }
  assert {
    condition = (
      toset(google_project_iam_custom_role.control_plane_image["image_lookup"].permissions) == toset(["artifactregistry.repositories.get", "artifactregistry.repositories.downloadArtifacts"])
      && toset(google_project_iam_custom_role.control_plane_image["image_publish"].permissions) == toset(["artifactregistry.repositories.get", "artifactregistry.repositories.downloadArtifacts", "artifactregistry.repositories.uploadArtifacts"])
      && alltrue([for key, grant in google_artifact_registry_repository_iam_member.control_plane_image :
        grant.project == "ciproxy" && grant.location == "us-central1" && grant.repository == "ciproxy-shared-runtime"
        && grant.member == "serviceAccount:ciproxy-${replace(key, "_", "-")}@ciproxy.iam.gserviceaccount.com"
        && grant.role == google_project_iam_custom_role.control_plane_image[key].name
      ]) && length(google_artifact_registry_repository_iam_member.control_plane_image) == 2
      && output.control_plane_registry_ready
    )
    error_message = "Lookup downloads only. Publish downloads/uploads only in the shared repository, with no deletion or IAM rights."
  }
  assert {
    condition = (
      toset(google_project_iam_custom_role.image_policy.permissions) == toset(["artifactregistry.repositories.getIamPolicy", "artifactregistry.repositories.setIamPolicy"])
      && toset(keys(google_artifact_registry_repository_iam_member.image_policy)) == toset(["trust_preview", "trust_production"])
      && alltrue([for key, grant in google_artifact_registry_repository_iam_member.image_policy :
        grant.project == "ciproxy" && grant.location == "us-central1" && grant.repository == "ciproxy-shared-runtime"
        && grant.member == (key == "trust_preview" ? "serviceAccount:ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com" : "serviceAccount:ciproxy-trust-prod-apply@ciproxy.iam.gserviceaccount.com")
        && grant.role == google_project_iam_custom_role.image_policy.name
      ])
    )
    error_message = "Only protected-main trust accounts maintain IAM on the exact shared repository."
  }
  assert {
    condition = alltrue([for key in ["image_lookup", "image_publish"] :
      strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.repository_id == '1442045837'")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.repository_owner_id == '307519338'")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.environment == 'main-preview'")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.event_name == 'push'")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "deploy-main-preview.yml@' + assertion.ref")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.ref in ['refs/heads/calummoore/ci-proxy-load-balancer', 'refs/heads/main']")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.job_workflow_ref == 'Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@refs/heads/calummoore/ci-proxy-load-balancer'")
      && lookup(google_iam_workload_identity_pool_provider.github[key].attribute_mapping, "attribute.job_workflow_ref", "") == "assertion.job_workflow_ref"
      && google_service_account_iam_member.federation[key].role == "roles/iam.workloadIdentityUser"
      && endswith(google_service_account_iam_member.federation[key].member, "/attribute.access/${key}")
      && google_service_account_iam_member.federation[key].service_account_id == google_service_account.ci[key].name
    ])
    error_message = "Image federation must admit only the trusted primary image workflow, preview push, environment, repository, owner, and exact pool/account."
  }
}
run "launch_keeps_preview_image_publish_available" {
  command = plan
  variables {
    production_enabled           = true
    control_plane_registry_ready = true
  }
  assert {
    condition = alltrue([for key in ["image_lookup", "image_publish"] :
      !google_service_account.ci[key].disabled
      && !google_iam_workload_identity_pool.github[key].disabled
      && !google_iam_workload_identity_pool_provider.github[key].disabled
    ])
    error_message = "Preview must build the image before production promotes that same digest after launch."
  }
}
