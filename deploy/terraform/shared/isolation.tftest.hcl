mock_provider "google" {
  mock_resource "google_compute_network" {
    defaults = { id = "projects/ciproxy/global/networks/ciproxy-shared" }
  }
}
variables {
  production_enabled = false
}
run "core_keeps_only_phase_selected_runtime_federation" {
  command = plan
  assert {
    condition     = toset(keys(google_service_account_iam_member.runtime)) == toset(["main-preview"])
    error_message = "Prelaunch core must install no production runtime impersonation grant."
  }
}
run "production_core_keeps_namespaces_separate" {
  command = plan
  variables { production_enabled = true }
  assert {
    condition     = google_service_account_iam_member.runtime["production"].member == "serviceAccount:ciproxy.svc.id.goog[ciproxy-production/control-plane]" && google_service_account_iam_member.runtime["main-preview"].member == "serviceAccount:ciproxy.svc.id.goog[ciproxy-main-preview/control-plane]"
    error_message = "One cluster keeps separate runtime federation namespaces."
  }
}
