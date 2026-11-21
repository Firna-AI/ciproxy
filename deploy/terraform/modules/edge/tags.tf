# The URL map name remains the invalidation argument. Tag bindings require
# its permanent numeric ID, which is available only after map creation.
resource "google_tags_tag_binding" "edge" {
  parent    = "//compute.googleapis.com/projects/${var.project_number}/global/urlMaps/${google_compute_url_map.https.map_id}"
  tag_value = var.environment_tag
}

output "tag_parent" { value = google_tags_tag_binding.edge.parent }
