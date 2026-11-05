locals {
  uptime_targets = merge({
    site      = { host = local.config.hosts.site, path = "/" }
    docs      = { host = local.config.hosts.site, path = "/docs/" }
    dashboard = { host = local.config.hosts.dashboard, path = "/" }
    releases  = { host = local.config.hosts.releases, path = "/ciproxy-agent" }
    }, var.control_plane_enabled ? {
    api = { host = local.config.hosts.dashboard, path = "/api/health/ready" }
  } : {})
}

resource "google_monitoring_uptime_check_config" "public" {
  for_each     = var.enable_uptime_checks ? local.uptime_targets : {}
  display_name = "${local.config.prefix}-${each.key}"
  timeout      = "10s"
  period       = "60s"
  http_check {
    path         = each.value.path
    port         = 443
    use_ssl      = true
    validate_ssl = true
  }
  monitored_resource {
    type = "uptime_url"
    labels = {
      project_id = local.config.project_id
      host       = each.value.host
    }
  }
  selected_regions = ["EUROPE", "USA", "ASIA_PACIFIC"]
}

resource "google_monitoring_alert_policy" "public" {
  for_each              = google_monitoring_uptime_check_config.public
  display_name          = "${local.config.prefix}-${each.key}-unavailable"
  combiner              = "OR"
  notification_channels = local.config.notification_channels
  enabled               = true
  conditions {
    display_name = "Two locations fail for five minutes"
    condition_threshold {
      filter = join(" AND ", [
        "metric.type=\"monitoring.googleapis.com/uptime_check/check_passed\"",
        "resource.type=\"uptime_url\"",
        "metric.label.check_id=\"${each.value.uptime_check_id}\"",
      ])
      comparison      = "COMPARISON_GT"
      threshold_value = 1
      duration        = "300s"
      aggregations {
        alignment_period     = "60s"
        per_series_aligner   = "ALIGN_NEXT_OLDER"
        cross_series_reducer = "REDUCE_COUNT_FALSE"
        group_by_fields      = ["resource.label.host"]
      }
      trigger {
        count = 1
      }
    }
  }
  alert_strategy {
    auto_close = "1800s"
  }
  documentation {
    mime_type = "text/markdown"
    content   = "Check DNS delegation, the managed certificate, the current deploy run, and backend health for https://${local.uptime_targets[each.key].host}${local.uptime_targets[each.key].path}."
  }
}

output "public_ip" {
  value = google_compute_global_address.public.address
}

output "url_map" {
  value = google_compute_url_map.https.name
}

output "image_backend" {
  value = google_compute_backend_bucket.images.name
}

output "readiness" {
  description = "Actual backend groups and uptime targets for operator inspection."
  value = {
    backend_groups = { for name, service in google_compute_backend_service.runtime : name => sort([for backend in service.backend : backend.group]) }
    uptime_hosts   = sort([for check in google_monitoring_uptime_check_config.public : check.monitored_resource[0].labels.host])
    channels       = { for name, policy in google_monitoring_alert_policy.public : name => policy.notification_channels }
  }
}
