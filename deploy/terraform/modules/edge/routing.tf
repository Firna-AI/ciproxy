resource "google_compute_url_map" "https" {
  name            = local.config.prefix
  default_service = google_compute_backend_bucket.public["site"].id
  host_rule {
    hosts        = [local.config.hosts.site]
    path_matcher = "site"
  }
  host_rule {
    hosts        = [local.config.hosts.dashboard]
    path_matcher = "dashboard"
  }
  host_rule {
    hosts        = [local.config.hosts.releases]
    path_matcher = "releases"
  }
  path_matcher {
    name            = "site"
    default_service = google_compute_backend_bucket.public["site"].id
    path_rule {
      # Generated hashes only. The more specific Pagefind child prefixes
      # override its mutable parent. HTML aliases keep the default.
      paths   = ["/_astro/*", "/pagefind/index/*", "/pagefind/fragment/*"]
      service = google_compute_backend_bucket.assets["site"].id
    }
    path_rule {
      paths   = ["/pagefind/*", "/favicon.svg", "/sitemap-index.xml", "/sitemap-0.xml"]
      service = google_compute_backend_bucket.mutable["site"].id
    }
    path_rule {
      paths   = ["/"]
      service = google_compute_backend_bucket.public["site"].id
      route_action {
        url_rewrite {
          path_prefix_rewrite = "/index.html"
        }
      }
    }
  }
  path_matcher {
    name            = "dashboard"
    default_service = google_compute_backend_bucket.public["dashboard"].id
    path_rule {
      # Keep Expo's build-owned static namespace, not all /_expo/*.
      paths   = ["/_expo/static/*", "/assets/*"]
      service = google_compute_backend_bucket.assets["dashboard"].id
    }
    path_rule {
      paths   = ["/_expo/.routes.json"]
      service = google_compute_backend_bucket.mutable["dashboard"].id
    }
    path_rule {
      paths   = ["/"]
      service = google_compute_backend_bucket.public["dashboard"].id
      route_action {
        url_rewrite {
          path_prefix_rewrite = "/index.html"
        }
      }
    }
    path_rule {
      paths   = ["/api", "/api/*", "/agent", "/agent/*", "/hooks", "/hooks/*"]
      service = google_compute_backend_service.runtime["control_plane"].id
    }
    path_rule {
      paths   = ["/images", "/images/*"]
      service = google_compute_backend_bucket.images.id
    }
  }
  path_matcher {
    name            = "releases"
    default_service = google_compute_backend_service.runtime["release"].id
  }
}

resource "google_compute_url_map" "http" {
  name = "${local.config.prefix}-http"
  default_url_redirect {
    https_redirect         = true
    redirect_response_code = "PERMANENT_REDIRECT"
    strip_query            = false
  }
}

resource "google_compute_global_address" "public" {
  name = local.config.prefix
}

resource "google_compute_managed_ssl_certificate" "public" {
  name = local.config.prefix
  managed {
    domains = [for host in local.config.hosts : host]
  }
}

resource "google_compute_ssl_policy" "public" {
  name            = local.config.prefix
  min_tls_version = "TLS_1_2"
  profile         = "MODERN"
}

resource "google_compute_target_https_proxy" "public" {
  name             = local.config.prefix
  url_map          = google_compute_url_map.https.id
  ssl_certificates = [google_compute_managed_ssl_certificate.public.id]
  ssl_policy       = google_compute_ssl_policy.public.id
}

resource "google_compute_target_http_proxy" "redirect" {
  name    = local.config.prefix
  url_map = google_compute_url_map.http.id
}

resource "google_compute_global_forwarding_rule" "https" {
  name                  = "${local.config.prefix}-https"
  ip_address            = google_compute_global_address.public.address
  target                = google_compute_target_https_proxy.public.id
  port_range            = "443"
  load_balancing_scheme = "EXTERNAL_MANAGED"
}

resource "google_compute_global_forwarding_rule" "http" {
  name                  = "${local.config.prefix}-http"
  ip_address            = google_compute_global_address.public.address
  target                = google_compute_target_http_proxy.redirect.id
  port_range            = "80"
  load_balancing_scheme = "EXTERNAL_MANAGED"
}

resource "google_dns_record_set" "public" {
  for_each     = local.config.dns_zones
  name         = "${local.config.hosts[each.key]}."
  managed_zone = each.value
  type         = "A"
  ttl          = 300
  rrdatas      = [google_compute_global_address.public.address]
}
