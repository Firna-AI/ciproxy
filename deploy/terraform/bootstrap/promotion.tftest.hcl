mock_provider "google" {
  mock_data "google_project" { defaults = { number = "123456789" } }
}
variables {
  project_id         = "ciproxy"
  production_enabled = false
}
run "prelaunch_accounts_have_no_environment_admin_escape" {
  command = apply
  assert {
    condition = (
      alltrue([for grant in google_project_iam_member.shared_plan : !contains(["roles/viewer", "roles/container.viewer", "roles/container.developer", "roles/artifactregistry.reader"], grant.role)])
      && !contains(google_project_iam_custom_role.plan_metadata.permissions, "container.secrets.get")
      && !contains(google_project_iam_custom_role.plan_metadata.permissions, "container.pods.getLogs")
      && !contains(google_project_iam_custom_role.plan_metadata.permissions, "container.clusters.getCredentials")
      && !contains(google_project_iam_custom_role.plan_metadata.permissions, "storage.objects.get")
      && !contains(google_project_iam_custom_role.plan_metadata.permissions, "artifactregistry.repositories.downloadArtifacts")
    )
    error_message = "Central plan reads metadata, never application objects, Kubernetes secrets/logs, or admin credentials."
  }
  assert {
    condition = (
      alltrue([for key, grant in google_project_iam_member.sql_runtime :
        grant.member == "serviceAccount:ciproxy-${split(":", key)[0]}-control@ciproxy.iam.gserviceaccount.com"
        && strcontains(grant.condition[0].expression, "instances/ciproxy-${split(":", key)[0]}'")
      ]) && google_project_iam_member.nodes.member == "serviceAccount:ciproxy-shared-gke@ciproxy.iam.gserviceaccount.com"
      && google_project_iam_member.registry_nodes.member == "serviceAccount:ciproxy-shared-gke@ciproxy.iam.gserviceaccount.com"
      && alltrue([for env in local.environments :
        google_project_iam_member.cluster_access[env].member == "serviceAccount:ciproxy-${env}-deploy@ciproxy.iam.gserviceaccount.com"
        && google_project_iam_member.operation_reader[env].member == "serviceAccount:ciproxy-${env}-deploy@ciproxy.iam.gserviceaccount.com"
        && google_project_iam_member.service_consumer["${env}-deploy"].member == "serviceAccount:ciproxy-${env}-deploy@ciproxy.iam.gserviceaccount.com"
      ])
    )
    error_message = "Runtime SQL, node, endpoint, consumer, and operation grants must bind their intended accounts."
  }
  assert {
    condition = (
      google_service_account_iam_member.runtime_policy["preview"].service_account_id == "projects/ciproxy/serviceAccounts/ciproxy-main-preview-control@ciproxy.iam.gserviceaccount.com"
      && google_service_account_iam_member.runtime_policy["preview"].member == "serviceAccount:ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
      && length([for grant in google_service_account_iam_member.runtime_policy : grant if grant.member == "serviceAccount:ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"]) == 1
      && toset(google_project_iam_custom_role.runtime_policy.permissions) == toset(["iam.serviceAccounts.getIamPolicy", "iam.serviceAccounts.setIamPolicy"])
    )
    error_message = "Prelaunch shared apply can change only the preview runtime policy, never production or trust."
  }
  assert {
    condition = (alltrue([for grant in google_project_iam_member.trust : contains([
      "serviceAccount:ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com",
      "serviceAccount:ciproxy-trust-prod-apply@ciproxy.iam.gserviceaccount.com",
      ], grant.member)]) && alltrue([for grant in merge(google_project_iam_member.shared_plan, google_project_iam_member.plan_metadata) : contains([
      "serviceAccount:ciproxy-shared-plan@ciproxy.iam.gserviceaccount.com",
      "serviceAccount:ciproxy-shared-prod-plan@ciproxy.iam.gserviceaccount.com",
      ], grant.member)])
    )
    error_message = "Central trust and plan grants must never bind an environment account."
  }
  assert {
    condition = (alltrue([for member in concat(
      [for g in google_project_iam_member.shared : g.member],
      [for g in google_project_iam_member.shared_api : g.member],
      [for g in google_project_iam_member.cluster_update : g.member],
      [google_project_iam_member.cluster_lifecycle.member],
      [for g in google_project_iam_member.shared_secret : g.member],
      [for g in google_project_iam_member.shared_kms : g.member],
      [for g in google_project_iam_member.bucket_metadata : g.member],
      [for g in google_project_iam_member.static_bucket_policy : g.member],
      [for g in google_project_iam_member.tag_binding : g.member],
      [for g in google_project_iam_member.registry : g.member],
      [google_project_iam_member.prelaunch.member],
      ) : contains([
        "serviceAccount:ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com",
        "serviceAccount:ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com",
      ], member)])
    )
    error_message = "Every current central infrastructure project grant must bind a central shared apply account."
  }
  assert {
    condition = (alltrue([for env in local.environments :
      google_storage_bucket_iam_member.writer["${env}:${env}-apply"].member == "serviceAccount:ciproxy-${env}-apply@ciproxy.iam.gserviceaccount.com"
      && length([for grant in google_storage_bucket_iam_member.writer : grant if startswith(grant.member, "serviceAccount:ciproxy-${env}-")]) == 1
      ]) && alltrue([for grant in google_storage_bucket_iam_member.trust : contains([
        "serviceAccount:ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com",
        "serviceAccount:ciproxy-trust-prod-apply@ciproxy.iam.gserviceaccount.com",
      ], grant.member)])
      && alltrue([for grant in google_storage_bucket_iam_member.trust_policy : contains([
        "serviceAccount:ciproxy-trust-preview-apply@ciproxy.iam.gserviceaccount.com",
        "serviceAccount:ciproxy-trust-prod-apply@ciproxy.iam.gserviceaccount.com",
      ], grant.member)])
    )
    error_message = "Each environment has one state writer on its own bucket. Trust bucket grants bind only trust accounts."
  }
  assert {
    condition = (toset(google_project_iam_custom_role.invalidate.permissions) == toset(["compute.urlMaps.invalidateCache"])
      && toset(google_project_iam_custom_role.operation_reader.permissions) == toset(["compute.globalOperations.get"])
      && alltrue([for env in local.environments :
        google_project_iam_member.invalidate[env].member == "serviceAccount:ciproxy-${env}-deploy@ciproxy.iam.gserviceaccount.com"
        && strcontains(google_project_iam_member.invalidate[env].condition[0].expression, "resource.matchTagId(")
      ])
    )
    error_message = "Own tagged CDN invalidation and untagged operation completion need separate grants."
  }
  assert {
    condition = (alltrue([for grant in google_project_iam_member.cluster_access : grant.role == google_project_iam_custom_role.cluster_access.name]) && toset(google_project_iam_custom_role.cluster_access.permissions) == toset(["container.clusters.get", "container.clusters.connect"])
    )
    error_message = "Environment deploy must not update namespace labels through IAM. Object writes require namespace RBAC."
  }
  assert {
    condition = (alltrue([for grant in google_project_iam_member.shared : !can(regex("iam\\.|IamAdmin|owner|editor$", grant.role)) || grant.role == "roles/monitoring.editor"])
      && !contains(google_project_iam_custom_role.shared_api.permissions, "container.clusters.getCredentials")
      && !contains(google_project_iam_custom_role.shared_api.permissions, "resourcemanager.projects.setIamPolicy")
    )
    error_message = "Preview-accessible shared apply must have no IAM administration or Kubernetes admin credentials."
  }
  assert {
    condition = (alltrue([for env in local.environments :
      !google_service_account.ci["${env}-apply"].disabled
      && google_storage_bucket_iam_member.writer["${env}:${env}-apply"].bucket == "ciproxy-${env}-tfstate-ciproxy"
      ]) && alltrue([for grant in google_project_iam_member.sql_runtime : strcontains(grant.condition[0].expression, "resource.type == 'sqladmin.googleapis.com/Instance'")])
    )
    error_message = "Environment state and runtime identity must stay in its own namespace and SQL instance."
  }
  assert {
    condition = (strcontains(google_iam_workload_identity_pool_provider.github["trust_preview"].attribute_condition, "assertion.ref == 'refs/heads/main'")
      && strcontains(google_iam_workload_identity_pool_provider.github["trust_preview"].attribute_condition, "deploy-apply.yml@' + assertion.ref")
      && !strcontains(google_iam_workload_identity_pool_provider.github["trust_preview"].attribute_condition, "calummoore/")
      && google_iam_workload_identity_pool_provider.github["shared_preview"].attribute_mapping["attribute.access"] == "'shared_preview'"
    )
    error_message = "Target-primary preview must never obtain the protected-main trust identity."
  }
  assert {
    condition = (!google_service_account.ci["shared_preview"].disabled && !google_service_account.ci["trust_preview"].disabled
      && alltrue([for grant in google_storage_bucket_iam_member.plan_lock : !strcontains(grant.condition[0].expression, "default.tfstate")])
      && toset(google_project_iam_custom_role.plan_lock.permissions) == toset(["storage.objects.create", "storage.objects.delete"])
    )
    error_message = "Prelaunch accounts are enabled. Plan writes only exact lock objects, never state."
  }
}
run "launch_permanently_retires_preview_shared_authority" {
  command = plan
  variables { production_enabled = true }
  assert {
    condition = (alltrue([for key in ["shared_plan", "shared_preview", "trust_preview"] :
      google_service_account.ci[key].disabled
      && google_iam_workload_identity_pool.github[key].disabled
      && google_iam_workload_identity_pool_provider.github[key].disabled
      ]) && !google_service_account.ci["shared_production"].disabled && !google_service_account.ci["trust_production"].disabled
    )
    error_message = "Launch disables preview accounts and federation, including previously issued account tokens."
  }
  assert {
    condition = (alltrue([for key in ["shared_production", "trust_production"] :
      strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.ref == 'refs/heads/main'")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "assertion.event_name == 'workflow_run'")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "deploy-production.yml@' + assertion.ref")
      && strcontains(google_iam_workload_identity_pool_provider.github[key].attribute_condition, "deploy-apply.yml@' + assertion.ref")
      ])
    )
    error_message = "Postlaunch central writes require the production main caller and reusable apply job."
  }
}

override_resource {
  target = google_service_account.ci["main-preview-plan"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-main-preview-plan@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-main-preview-plan@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["main-preview-apply"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-main-preview-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-main-preview-apply@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["main-preview-deploy"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-main-preview-deploy@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-main-preview-deploy@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["production-plan"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-production-plan@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-production-plan@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["production-apply"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-production-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-production-apply@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["production-deploy"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-production-deploy@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-production-deploy@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["shared_plan"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-shared-plan@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-shared-plan@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["shared_plan_production"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-shared-prod-plan@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-shared-prod-plan@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["shared_preview"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-shared-preview-apply@ciproxy.iam.gserviceaccount.com"
  }
}

override_resource {
  target = google_service_account.ci["shared_production"]
  values = {
    name  = "projects/ciproxy/serviceAccounts/ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
    email = "ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
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

override_resource {
  target = google_service_account.runtime["main-preview"]
  values = { name = "projects/ciproxy/serviceAccounts/ciproxy-main-preview-control@ciproxy.iam.gserviceaccount.com", email = "ciproxy-main-preview-control@ciproxy.iam.gserviceaccount.com" }
}

override_resource {
  target = google_service_account.runtime["production"]
  values = { name = "projects/ciproxy/serviceAccounts/ciproxy-production-control@ciproxy.iam.gserviceaccount.com", email = "ciproxy-production-control@ciproxy.iam.gserviceaccount.com" }
}

override_resource {
  target = google_service_account.nodes
  values = { name = "projects/ciproxy/serviceAccounts/ciproxy-shared-gke@ciproxy.iam.gserviceaccount.com", email = "ciproxy-shared-gke@ciproxy.iam.gserviceaccount.com" }
}
