# Keep historical entries while immutable rollout evidence can still select
# them. Removing a zone from desired placement must not invalidate old evidence.
locals {
  desired = ["europe-west1-b", "europe-west1-c", "europe-west1-d"]
  catalog = {
    europe-west1-b = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-b"
    europe-west1-c = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-c"
    europe-west1-d = "https://www.googleapis.com/compute/v1/projects/ciproxy/zones/europe-west1-d"
  }
}
output "allowed_zones" { value = local.desired }
output "catalog" { value = local.catalog }
