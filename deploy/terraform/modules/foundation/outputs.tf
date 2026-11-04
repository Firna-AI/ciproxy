output "deployment" {
  value = {
    project_id            = var.project_id
    environment           = var.environment
    region                = local.region
    allowed_zones         = local.allowed_zones
    prefix                = local.prefix
    cluster_name          = google_container_cluster.main.name
    runtime_repository    = "${local.region}-docker.pkg.dev/${var.project_id}/${google_artifact_registry_repository.runtime.repository_id}"
    buckets               = { for key, bucket in google_storage_bucket.public : key => bucket.name }
    images_bucket         = google_storage_bucket.images.name
    hosts                 = local.hosts
    release_secret        = google_secret_manager_secret.release.secret_id
    control_plane_account = google_service_account.control_plane.email
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
      for key, host in local.hosts : key => google_dns_managed_zone.public[
        var.environment == "production" ? "site" : key
      ].name
    }
    buckets               = { for key, bucket in google_storage_bucket.public : key => bucket.name }
    images_bucket         = google_storage_bucket.images.name
    notification_channels = [for channel in google_monitoring_notification_channel.email : channel.name]
  }
}

output "dns_name_servers" {
  value = {
    for zone in google_dns_managed_zone.public : zone.dns_name => zone.name_servers
  }
}
