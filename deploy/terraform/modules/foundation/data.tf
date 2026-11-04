resource "google_service_account" "control_plane" {
  account_id   = "${local.prefix}-control"
  display_name = "ciproxy control plane"
  depends_on   = [google_project_service.service]
}

resource "google_service_account_iam_member" "control_plane" {
  service_account_id = google_service_account.control_plane.name
  role               = "roles/iam.workloadIdentityUser"
  member             = "serviceAccount:${var.project_id}.svc.id.goog[ciproxy/control-plane]"
  depends_on         = [google_container_cluster.main]
}

resource "google_project_iam_member" "sql_client" {
  for_each = toset(["roles/cloudsql.client", "roles/cloudsql.instanceUser"])
  project  = var.project_id
  role     = each.value
  member   = "serviceAccount:${google_service_account.control_plane.email}"
}

resource "google_sql_database_instance" "main" {
  name                = local.prefix
  region              = local.region
  database_version    = "POSTGRES_17"
  deletion_protection = true
  settings {
    tier              = var.environment == "production" ? "db-custom-2-7680" : "db-custom-1-3840"
    availability_type = var.environment == "production" ? "REGIONAL" : "ZONAL"
    disk_type         = "PD_SSD"
    disk_size         = 20
    disk_autoresize   = true
    edition           = "ENTERPRISE"
    ip_configuration {
      ipv4_enabled    = false
      private_network = google_compute_network.main.id
      ssl_mode        = "ENCRYPTED_ONLY"
    }
    database_flags {
      name  = "cloudsql.iam_authentication"
      value = "on"
    }
    backup_configuration {
      enabled                        = true
      point_in_time_recovery_enabled = true
      start_time                     = "02:00"
      transaction_log_retention_days = 7
      backup_retention_settings {
        retained_backups = 14
      }
    }
    maintenance_window {
      day          = 7
      hour         = 3
      update_track = "stable"
    }
  }
  depends_on = [google_service_networking_connection.sql]
}

resource "google_sql_database" "main" {
  name     = "ciproxy"
  instance = google_sql_database_instance.main.name
}

# Cloud SQL IAM authentication supplies short-lived passwords through the
# proxy. No database password is generated or stored by Terraform.
resource "google_sql_user" "control_plane" {
  name     = trimsuffix(google_service_account.control_plane.email, ".gserviceaccount.com")
  instance = google_sql_database_instance.main.name
  type     = "CLOUD_IAM_SERVICE_ACCOUNT"
}

resource "google_kms_key_ring" "main" {
  name       = local.prefix
  location   = local.region
  depends_on = [google_project_service.service]
}

resource "google_kms_crypto_key" "boat" {
  name            = "boat-account-keys"
  key_ring        = google_kms_key_ring.main.id
  rotation_period = "7776000s"
  lifecycle {
    prevent_destroy = true
  }
}

resource "google_kms_crypto_key_iam_member" "control_plane" {
  crypto_key_id = google_kms_crypto_key.boat.id
  role          = "roles/cloudkms.cryptoKeyEncrypterDecrypter"
  member        = "serviceAccount:${google_service_account.control_plane.email}"
}
