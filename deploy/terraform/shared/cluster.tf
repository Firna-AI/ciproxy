resource "google_artifact_registry_repository" "runtime" {
  location      = local.region
  repository_id = "${local.prefix}-runtime"
  format        = "DOCKER"
  depends_on    = [google_project_service.service]
}

resource "google_container_cluster" "main" {
  name             = local.prefix
  location         = local.region
  enable_autopilot = true
  # NEG pre-provisioning needs 1.36.2-gke.3104000 or newer. This alias
  # selects an available 1.36.4 GKE patch while Regular keeps upgrades on.
  min_master_version  = "1.36.4"
  deletion_protection = true
  network             = google_compute_network.main.id
  subnetwork          = google_compute_subnetwork.main.id
  release_channel {
    channel = "REGULAR"
  }
  ip_allocation_policy {
    cluster_secondary_range_name  = "pods"
    services_secondary_range_name = "services"
  }
  private_cluster_config {
    enable_private_nodes = true
    # GKE reports a private endpoint when the IP endpoints below are off.
    # Stating it keeps each plan free of a change to the control plane access.
    enable_private_endpoint = true
  }
  control_plane_endpoints_config {
    dns_endpoint_config {
      allow_external_traffic = true
    }
    ip_endpoints_config {
      enabled = false
    }
  }
  cluster_autoscaling {
    auto_provisioning_defaults {
      service_account = local.node_account
      oauth_scopes    = ["https://www.googleapis.com/auth/cloud-platform"]
    }
  }
  workload_identity_config {
    workload_pool = "ciproxy.svc.id.goog"
  }
  secret_manager_config {
    enabled = true
    # Roll out pods after secret updates. Do not refresh a running pod's
    # signing key while CDN_SIGNING_KEY_NAME remains fixed in its env.
  }
  depends_on = [google_project_service.service]
}
