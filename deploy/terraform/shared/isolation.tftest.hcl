mock_provider "google" {
  mock_resource "google_compute_network" {
    defaults = { id = "projects/ciproxy/global/networks/ciproxy-shared" }
  }
}
variables {
  production_enabled = false
}
override_data {
  target = data.terraform_remote_state.trust
  values = { outputs = {
    project_number = "123456789"
    runtime_service_account_uids = {
      main-preview = "11111111-1111-4111-8111-111111111111"
      production   = "22222222-2222-4222-8222-222222222222"
    }
  } }
}
run "first_operator_core_needs_no_ksa_or_runtime_binding" {
  command = plan
  override_data {
    target = data.terraform_remote_state.trust
    values = { outputs = { project_number = "123456789", runtime_service_account_uids = {} } }
  }
  assert {
    condition     = length(google_service_account_iam_member.runtime) == 0 && google_container_cluster.main.enable_autopilot
    error_message = "The operator creates the intended Autopilot cluster before any UID or runtime impersonation exists."
  }
}
run "core_keeps_only_phase_selected_runtime_federation" {
  command = plan
  assert {
    condition     = toset(keys(google_service_account_iam_member.runtime)) == toset(["main-preview"]) && google_service_account_iam_member.runtime["main-preview"].member == "principal://iam.googleapis.com/projects/123456789/locations/global/workloadIdentityPools/ciproxy.svc.id.goog/kubernetes.serviceaccount.uid/11111111-1111-4111-8111-111111111111" && google_service_account_iam_member.runtime["main-preview"].service_account_id == "projects/ciproxy/serviceAccounts/ciproxy-main-preview-control@ciproxy.iam.gserviceaccount.com"
    error_message = "Prelaunch core must install no production runtime impersonation grant."
  }
}
run "production_core_keeps_namespaces_separate" {
  command = plan
  variables { production_enabled = true }
  assert {
    condition = (
      google_service_account_iam_member.runtime["production"].member == "principal://iam.googleapis.com/projects/123456789/locations/global/workloadIdentityPools/ciproxy.svc.id.goog/kubernetes.serviceaccount.uid/22222222-2222-4222-8222-222222222222"
      && google_service_account_iam_member.runtime["main-preview"].member == "principal://iam.googleapis.com/projects/123456789/locations/global/workloadIdentityPools/ciproxy.svc.id.goog/kubernetes.serviceaccount.uid/11111111-1111-4111-8111-111111111111"
      && alltrue([for env, member in google_service_account_iam_member.runtime :
        member.service_account_id == "projects/ciproxy/serviceAccounts/ciproxy-${env}-control@ciproxy.iam.gserviceaccount.com"
        && member.role == "roles/iam.workloadIdentityUser" && !strcontains(member.member, "/subject/ns/") && !strcontains(member.member, "svc.id.goog[")
      ])
    )
    error_message = "Only trusted operator-captured UIDs may impersonate runtime accounts. Namespace/name members must never return."
  }
}
