output "deployment" {
  value = {
    project_id            = var.project_id
    namespace             = local.prefix
    environment           = var.environment
    region                = local.region
    allowed_zones         = local.allowed_zones
    prefix                = local.prefix
    cluster_name          = var.shared.cluster_name
    runtime_repository    = var.shared.runtime_repository
    buckets               = { for key, bucket in google_storage_bucket.public : key => bucket.name }
    images_bucket         = google_storage_bucket.images.name
    hosts                 = local.hosts
    release_secret        = google_secret_manager_secret.release.secret_id
    control_plane_account = local.control_account
    sql_instance          = google_sql_database_instance.main.name
    sql_connection        = google_sql_database_instance.main.connection_name
    sql_user              = google_sql_user.control_plane.name
    kms_key               = google_kms_crypto_key.boat.id
    secrets = {
      for key, secret in google_secret_manager_secret.control_plane : key => secret.id
    }
  }
}

output "edge" {
  value = {
    project_id    = var.project_id
    environment   = var.environment
    prefix        = local.prefix
    allowed_zones = local.allowed_zones
    hosts         = local.hosts
    dns_zones = {
      for key, zone in google_dns_managed_zone.public : key => zone.name
    }
    buckets               = { for key, bucket in google_storage_bucket.public : key => bucket.name }
    images_bucket         = google_storage_bucket.images.name
    notification_channels = []
  }
}

output "dns_name_servers" {
  value = {
    for zone in google_dns_managed_zone.public : zone.dns_name => zone.name_servers
  }
}
