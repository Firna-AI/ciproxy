mock_provider "google" {}
variables {
  project_number        = "123456789"
  environment_tag       = "tagValues/1"
  zone_map              = { europe-west1-b = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b" }
  control_plane_enabled = true
  configuration = {
    project_id    = "ciproxy"
    environment   = "production"
    prefix        = "ciproxy-production"
    allowed_zones = ["europe-west1-b"]
    hosts = {
      site      = "ciproxy.dev"
      dashboard = "app.ciproxy.dev"
      releases  = "get.ciproxy.dev"
    }
    dns_zones = {
      dashboard = "ciproxy-production-dashboard"
      releases  = "ciproxy-production-releases"
    }
    buckets = {
      site      = "ciproxy-production-site-ciproxy"
      dashboard = "ciproxy-production-dashboard-ciproxy"
    }
    images_bucket         = "ciproxy-production-images-ciproxy"
    notification_channels = []
  }
}
override_resource {
  target          = google_compute_backend_bucket.public["site"]
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/site-html" }
}
override_resource {
  target          = google_compute_backend_bucket.public["dashboard"]
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/dashboard-html" }
}
override_resource {
  target          = google_compute_backend_bucket.assets["site"]
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/site-assets" }
}
override_resource {
  target          = google_compute_backend_bucket.assets["dashboard"]
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/dashboard-assets" }
}
override_resource {
  target          = google_compute_backend_bucket.images
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/signed-images" }
}
override_resource {
  target          = google_compute_backend_service.runtime["control_plane"]
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendServices/control" }
}
override_resource {
  target          = google_compute_backend_service.runtime["release"]
  override_during = plan
  values          = { id = "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendServices/release" }
}

run "private_assets_cache_without_caching_html_or_reserved_routes" {
  command = plan
  module { source = "../modules/edge" }
  assert {
    condition = (
      alltrue([for backend in google_compute_backend_bucket.assets :
        backend.enable_cdn && backend.cdn_policy[0].cache_mode == "FORCE_CACHE_ALL"
        && backend.cdn_policy[0].default_ttl == 31536000
        && backend.cdn_policy[0].max_ttl == 31536000
        && backend.cdn_policy[0].client_ttl == 86400
        && !backend.cdn_policy[0].negative_caching
        && backend.cdn_policy[0].serve_while_stale == 0
      ])
      && alltrue([for backend in google_compute_backend_bucket.public :
        backend.enable_cdn && backend.cdn_policy[0].cache_mode == "USE_ORIGIN_HEADERS"
        && !backend.cdn_policy[0].negative_caching
        && backend.cdn_policy[0].serve_while_stale == 0
      ])
    )
    error_message = "Private assets require forced one-year CDN caching and a one-day browser cap. HTML/no-store must never be force-cached."
  }
  assert {
    condition = (
      google_compute_url_map.https.default_service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/site-html"
      && alltrue([for matcher in google_compute_url_map.https.path_matcher :
        matcher.name == "releases"
        ? matcher.default_service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendServices/release"
        : matcher.default_service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/${matcher.name}-html"
      ])
      && alltrue([for matcher in google_compute_url_map.https.path_matcher :
        anytrue([for rule in matcher.path_rule :
          toset(rule.paths) == toset(["/"])
          && rule.service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/${matcher.name}-html"
          && try(rule.route_action[0].url_rewrite[0].path_prefix_rewrite, "") == "/index.html"
        ]) if matcher.name != "releases"
      ])
    )
    error_message = "Static defaults and root rewrites must use HTML wrappers. Release requests must keep the release backend."
  }
  assert {
    condition = alltrue([for matcher in google_compute_url_map.https.path_matcher :
      toset(flatten([for rule in matcher.path_rule : rule.paths
        if rule.service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/${matcher.name}-assets"
        ])) == (matcher.name == "site"
        ? toset(["/_astro/*", "/pagefind/*", "/favicon.svg", "/sitemap-index.xml", "/sitemap-0.xml"])
        : toset(["/_expo/static/*", "/_expo/.routes.json", "/assets/*"])
      ) if matcher.name != "releases"
    ])
    error_message = "Only concrete build asset prefixes and root files may select the cached wrappers. Docs, HTML aliases, and other paths use defaults."
  }
  assert {
    condition = (
      toset(flatten([for matcher in google_compute_url_map.https.path_matcher : [for rule in matcher.path_rule : rule.paths
        if rule.service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendServices/control"
      ] if matcher.name == "dashboard"])) == toset(["/api", "/api/*", "/agent", "/agent/*", "/hooks", "/hooks/*"])
      && toset(flatten([for matcher in google_compute_url_map.https.path_matcher : [for rule in matcher.path_rule : rule.paths
        if rule.service == "https://www.googleapis.com/compute/v1/projects/ciproxy/global/backendBuckets/signed-images"
      ] if matcher.name == "dashboard"])) == toset(["/images", "/images/*"])
      && alltrue([for backend in google_compute_backend_service.runtime : !backend.enable_cdn])
      && google_compute_backend_bucket.images.cdn_policy[0].cache_mode == "USE_ORIGIN_HEADERS"
      && google_compute_backend_bucket.images.cdn_policy[0].signed_url_cache_max_age_sec == 3600
    )
    error_message = "API/agent/hooks and signed image paths must retain their separate backends and cache policies."
  }
}
