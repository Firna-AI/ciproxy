variable "configuration" {
  type = object({
    project_id            = string
    environment           = string
    prefix                = string
    allowed_zones         = list(string)
    hosts                 = map(string)
    dns_zones             = map(string)
    buckets               = map(string)
    images_bucket         = string
    notification_channels = list(string)
  })
}

variable "control_plane_enabled" {
  description = "True only after a real HTTP runtime replaces the current CLI skeleton."
  type        = bool
  default     = false
}

variable "enable_uptime_checks" {
  description = "Enable after DNS delegation and certificate provisioning."
  type        = bool
  default     = false
}

locals {
  config = var.configuration
  runtimes = {
    release = {
      neg_name    = "${local.config.prefix}-release"
      zones       = local.config.allowed_zones
      health_path = "/"
      health_host = local.config.hosts.releases
    }
    control_plane = {
      neg_name    = "${local.config.prefix}-control"
      zones       = var.control_plane_enabled ? local.config.allowed_zones : []
      health_path = "/api/health/ready"
      health_host = local.config.hosts.dashboard
    }
  }
}

variable "zone_map" {
  description = "Trusted canonical zone paths. Evidence selects keys, never URL text."
  type        = map(string)
}
variable "project_number" { type = string }
variable "environment_tag" { type = string }
