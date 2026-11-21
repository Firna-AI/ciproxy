resource "google_compute_network" "main" {
  name                    = local.prefix
  auto_create_subnetworks = false
  depends_on              = [google_project_service.service]
}

resource "google_compute_subnetwork" "main" {
  name                     = local.prefix
  region                   = local.region
  network                  = google_compute_network.main.id
  ip_cidr_range            = "10.80.0.0/20"
  private_ip_google_access = true
  secondary_ip_range {
    range_name    = "pods"
    ip_cidr_range = "10.84.0.0/14"
  }
  secondary_ip_range {
    range_name    = "services"
    ip_cidr_range = "10.88.0.0/20"
  }
}

resource "google_compute_router" "main" {
  name    = local.prefix
  region  = local.region
  network = google_compute_network.main.id
}

resource "google_compute_router_nat" "egress" {
  name                               = local.prefix
  region                             = local.region
  router                             = google_compute_router.main.name
  nat_ip_allocate_option             = "AUTO_ONLY"
  source_subnetwork_ip_ranges_to_nat = "ALL_SUBNETWORKS_ALL_IP_RANGES"
}

resource "google_compute_global_address" "sql" {
  name          = "${local.prefix}-sql"
  purpose       = "VPC_PEERING"
  address_type  = "INTERNAL"
  prefix_length = 16
  network       = google_compute_network.main.id
}

resource "google_service_networking_connection" "sql" {
  network                 = google_compute_network.main.id
  service                 = "servicenetworking.googleapis.com"
  reserved_peering_ranges = [google_compute_global_address.sql.name]
}

# Global Application Load Balancer proxies and health probes reach pod NEGs.
resource "google_compute_firewall" "load_balancer" {
  name                    = "${local.prefix}-load-balancer"
  network                 = google_compute_network.main.name
  source_ranges           = ["35.191.0.0/16", "130.211.0.0/22"]
  target_service_accounts = [local.node_account]
  allow {
    protocol = "tcp"
    ports    = ["8080"]
  }
}
