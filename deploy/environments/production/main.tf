terraform {
  required_version = ">= 1.17.1, < 2.0.0"
  backend "gcs" {}
}
# This small root records the ready rollout for central edge promotion. The
# environment apply account can write only this environment's snapshot state.
variable "control_plane_enabled" { type = bool }
variable "enable_uptime_checks" { type = bool }
variable "notification_emails" { type = set(string) }
variable "allowed_zones" { type = list(string) }
variable "source_sha" { type = string }
output "runtime_flags" {
  value = {
    control_plane_enabled = var.control_plane_enabled
    enable_uptime_checks  = var.enable_uptime_checks
  }
}
output "notification_emails" { value = var.notification_emails }
output "allowed_zones" { value = var.allowed_zones }
output "source_sha" { value = var.source_sha }
