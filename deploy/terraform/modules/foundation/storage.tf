resource "google_storage_bucket" "public" {
  for_each                    = toset(["site", "dashboard"])
  name                        = "${local.prefix}-${each.key}-${var.project_id}"
  location                    = local.region
  uniform_bucket_level_access = true
  force_destroy               = false
  website {
    main_page_suffix = "index.html"
    not_found_page   = "404.html"
  }
  versioning {
    enabled = true
  }
}

resource "google_storage_bucket_iam_member" "public" {
  for_each = google_storage_bucket.public
  bucket   = each.value.name
  role     = "roles/storage.objectViewer"
  member   = "allUsers"
}

resource "google_storage_bucket_iam_member" "publisher" {
  for_each = google_storage_bucket.public
  bucket   = each.value.name
  role     = "roles/storage.objectAdmin"
  member   = "serviceAccount:${local.ci_email}"
}

resource "google_storage_bucket" "images" {
  name                        = "${local.prefix}-images-${var.project_id}"
  location                    = local.region
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false
  versioning {
    enabled = true
  }
}

# No allUsers grant exists on the image bucket. The signing setup script
# grants the CDN cache-fill identity access after a key creates that identity.
# Image publishing requires a separate operator, not the web deploy identity.

resource "google_secret_manager_secret" "control_plane" {
  for_each            = local.control_plane_secrets
  secret_id           = "${local.prefix}-${each.value}"
  deletion_protection = true
  replication {
    user_managed {
      replicas {
        location = local.region
      }
    }
  }
}

resource "google_secret_manager_secret_iam_member" "control_plane" {
  for_each  = google_secret_manager_secret.control_plane
  secret_id = each.value.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${local.control_account}"
}

resource "google_secret_manager_secret" "release" {
  secret_id           = "${local.prefix}-release-github-token"
  deletion_protection = true
  replication {
    user_managed {
      replicas {
        location = local.region
      }
    }
  }
}

resource "google_secret_manager_secret_iam_member" "release_sync" {
  secret_id = google_secret_manager_secret.release.id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${local.ci_email}"
}

resource "google_dns_managed_zone" "public" {
  for_each    = local.dns_domains
  name        = each.key == "site" ? local.prefix : "${local.prefix}-${each.key}"
  dns_name    = "${each.value}."
  description = "Delegate this zone before enabling public uptime checks."
}

# app/get become authoritative parents after Cloudflare delegates them.
# Read the named Google zones, never mutable rollout or prelaunch state.
data "google_dns_managed_zone" "preview" {
  for_each = local.preview_zones
  name     = each.value
}

resource "google_dns_record_set" "preview_delegation" {
  for_each     = local.preview_zones
  name         = "preview.${local.dns_domains[each.key]}."
  managed_zone = google_dns_managed_zone.public[each.key].name
  type         = "NS"
  ttl          = 3600
  rrdatas      = data.google_dns_managed_zone.preview[each.key].name_servers
}
