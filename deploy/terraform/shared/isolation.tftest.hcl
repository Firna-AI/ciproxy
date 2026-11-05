mock_provider "google" {
  mock_resource "google_compute_network" {
    defaults = { id = "projects/ciproxy/global/networks/ciproxy-shared" }
  }
}
variables {
  production_enabled = false
}
run "prelaunch_creates_no_production_resources" {
  command = plan
  assert {
    condition = (toset(keys(module.environment)) == toset(["main-preview"])
    )
    error_message = "Prelaunch shared infrastructure must not create production resources."
  }
  assert {
    condition     = toset(keys(google_service_account_iam_member.runtime)) == toset(["main-preview"]) && google_service_account_iam_member.runtime["main-preview"].member == "serviceAccount:ciproxy.svc.id.goog[ciproxy-main-preview/control-plane]"
    error_message = "Runtime federation follows cluster creation and keeps the preview namespace boundary."
  }

}
run "launch_keeps_data_and_namespaces_separate" {
  command = apply
  variables { production_enabled = true }
  assert {
    condition = (module.environment["main-preview"].deployment.namespace == "ciproxy-main-preview"
      && module.environment["production"].deployment.namespace == "ciproxy-production"
      && module.environment["main-preview"].deployment.prefix != module.environment["production"].deployment.prefix
      && module.environment["main-preview"].deployment.sql_instance == "ciproxy-main-preview"
      && module.environment["production"].deployment.sql_instance == "ciproxy-production"
    )
    error_message = "The shared owner must keep separate environment namespaces and SQL instances."
  }
}
