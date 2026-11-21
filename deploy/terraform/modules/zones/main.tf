# Keep historical entries while immutable rollout evidence can still select
# them. Removing a zone from desired placement must not invalidate old evidence.
locals {
  desired = ["us-central1-a", "us-central1-b", "us-central1-c"]
  catalog = {
    us-central1-a = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/us-central1-a"
    us-central1-b = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/us-central1-b"
    us-central1-c = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/us-central1-c"
  }
}
output "allowed_zones" { value = local.desired }
output "catalog" { value = local.catalog }
