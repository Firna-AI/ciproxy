mock_provider "google" {}
variables {
  project_id         = "ciproxy"
  production_enabled = false
}
run "prelaunch_updates_only_operator_cluster_without_creation_or_node_attachment" {
  command = plan
  assert {
    condition = (
      toset(google_project_iam_custom_role.cluster_update.permissions) == toset(["container.clusters.get", "container.clusters.update"])
      && alltrue([for grant in google_project_iam_member.cluster_update :
        grant.condition[0].expression == "resource.type == 'container.googleapis.com/Cluster' && resource.name == 'projects/ciproxy/locations/us-central1/clusters/ciproxy-shared'"
      ])
      && !anytrue([for permission in google_project_iam_custom_role.shared_api.permissions : startswith(permission, "container.clusters.")])
      && toset(keys(google_service_account_iam_member.node_user)) == toset(["shared_production"])
      && google_service_account_iam_member.node_user["shared_production"].member == "serviceAccount:ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
      && google_project_iam_member.cluster_lifecycle.member == "serviceAccount:ciproxy-shared-prod-apply@ciproxy.iam.gserviceaccount.com"
      && toset(google_project_iam_custom_role.cluster_lifecycle.permissions) == toset(["container.clusters.create", "container.clusters.delete"])
    )
    error_message = "Prelaunch must have no cluster create/delete, unconditioned cluster update, or node actAs. Only protected production retains lifecycle."
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
