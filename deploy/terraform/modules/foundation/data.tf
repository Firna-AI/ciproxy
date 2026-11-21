resource "google_sql_database_instance" "main" {
  name                = local.prefix
  region              = local.region
  database_version    = "POSTGRES_17"
  deletion_protection = true
  settings {
    deletion_protection_enabled = true
    tier                        = var.environment == "production" ? "db-custom-2-7680" : "db-custom-1-3840"
    availability_type           = var.environment == "production" ? "REGIONAL" : "ZONAL"
    disk_type                   = "PD_SSD"
    disk_size                   = 20
    disk_autoresize             = true
    edition                     = "ENTERPRISE"
    ip_configuration {
      ipv4_enabled    = false
      private_network = var.shared.network_id
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
}

resource "google_sql_database" "main" {
  name     = "ciproxy"
  instance = google_sql_database_instance.main.name
}

# Cloud SQL IAM authentication supplies short-lived passwords through the
# proxy. No database password is generated or stored by Terraform.
resource "google_sql_user" "control_plane" {
  name     = trimsuffix(local.control_account, ".gserviceaccount.com")
  instance = google_sql_database_instance.main.name
  type     = "CLOUD_IAM_SERVICE_ACCOUNT"
}

resource "google_kms_key_ring" "main" {
  name     = local.prefix
  location = local.region
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
  member        = "serviceAccount:${local.control_account}"
}
