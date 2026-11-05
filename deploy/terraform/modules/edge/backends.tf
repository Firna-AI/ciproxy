resource "google_compute_backend_bucket" "public" {
  for_each    = local.config.buckets
  name        = "${local.config.prefix}-${each.key}"
  bucket_name = each.value
  enable_cdn  = true
  cdn_policy {
    # Keep private origin delivery, but never force-cache HTML/no-store.
    cache_mode        = "USE_ORIGIN_HEADERS"
    serve_while_stale = 0
    negative_caching  = false
  }
}

resource "google_compute_backend_bucket" "assets" {
  for_each    = local.config.buckets
  name        = "${local.config.prefix}-${each.key}-assets"
  bucket_name = each.value
  enable_cdn  = true
  cdn_policy {
    # Private-bucket caching requires FORCE_CACHE_ALL. URL paths select
    # assets only. Every deploy invalidates stable names as well as hashes.
    cache_mode        = "FORCE_CACHE_ALL"
    default_ttl       = 31536000
    max_ttl           = 31536000
    client_ttl        = 86400
    serve_while_stale = 0
    negative_caching  = false
  }
}

# Google creates this service agent only after the first backend bucket.
# Iterate the cached wrappers to wait for their creation, once per origin.
resource "google_storage_bucket_iam_member" "origin_reader" {
  for_each = google_compute_backend_bucket.assets
  bucket   = each.value.bucket_name
  role     = "roles/storage.objectViewer"
  member   = "serviceAccount:service-${var.project_number}@https-lb.iam.gserviceaccount.com"
}

resource "google_compute_backend_bucket" "images" {
  name        = "${local.config.prefix}-images"
  bucket_name = local.config.images_bucket
  enable_cdn  = true
  cdn_policy {
    cache_mode                   = "USE_ORIGIN_HEADERS"
    signed_url_cache_max_age_sec = 3600
    serve_while_stale            = 0
    negative_caching             = false
  }
  # A local operator registers signing keys with register-cdn-key.sh.
  # Terraform never reads, stores, or outputs a signing key.
}

resource "google_compute_health_check" "runtime" {
  for_each            = local.runtimes
  name                = "${local.config.prefix}-${replace(each.key, "_", "-")}"
  check_interval_sec  = 10
  timeout_sec         = 5
  healthy_threshold   = 2
  unhealthy_threshold = 3
  http_health_check {
    port         = 8080
    request_path = each.value.health_path
    host         = each.value.health_host
  }
}

resource "google_compute_backend_service" "runtime" {
  for_each                        = local.runtimes
  name                            = "${local.config.prefix}-${replace(each.key, "_", "-")}"
  protocol                        = "HTTP"
  load_balancing_scheme           = "EXTERNAL_MANAGED"
  timeout_sec                     = 86400
  connection_draining_timeout_sec = 300
  health_checks                   = [google_compute_health_check.runtime[each.key].id]
  enable_cdn                      = false
  dynamic "backend" {
    for_each = toset(each.value.zones)
    content {
      # GKE pre-provisions this complete zone set before edge deployment.
      # Required pod affinity uses the same set, so later zone changes
      # cannot put a runtime in a zone without an attached backend.
      group                 = "${var.zone_map[backend.value]}/networkEndpointGroups/${each.value.neg_name}"
      balancing_mode        = "RATE"
      max_rate_per_endpoint = 100
    }
  }
  log_config {
    enable      = true
    sample_rate = 1
  }
}
