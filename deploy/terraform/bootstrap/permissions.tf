# Only central trust accounts have IAM administration. They are effectively
# owners. Their providers require protected main and the reusable apply job.
locals {
  trust_roles = toset([
    "roles/serviceusage.serviceUsageAdmin", "roles/resourcemanager.projectIamAdmin",
    "roles/iam.serviceAccountAdmin", "roles/iam.workloadIdentityPoolAdmin",
    "roles/iam.roleAdmin", "roles/resourcemanager.tagAdmin",
  ])
  shared_accounts = toset(["shared_preview", "shared_production"])
  account_environments = {
    shared_preview    = ["main-preview"]
    shared_production = ["main-preview", "production"]
  }
}
resource "google_project_iam_member" "trust" {
  for_each = { for pair in setproduct(["trust_preview", "trust_production"], local.trust_roles) : "${pair[0]}:${pair[1]}" => pair }
  project  = var.project_id
  role     = each.value[1]
  member   = "serviceAccount:${google_service_account.ci[each.value[0]].email}"
}
resource "google_project_iam_member" "shared_plan" {
  for_each = { for pair in setproduct(local.plan_accounts, [
    "roles/dns.reader", "roles/resourcemanager.tagViewer",
    "roles/serviceusage.serviceUsageViewer", "roles/serviceusage.serviceUsageConsumer",
    "roles/iam.securityReviewer", "roles/iam.roleViewer",
    "roles/iam.serviceAccountViewer", "roles/iam.workloadIdentityPoolViewer",
    "roles/secretmanager.viewer", "roles/cloudkms.viewer",
  ]) : "${pair[0]}:${pair[1]}" => pair }
  project = var.project_id
  role    = each.value[1]
  member  = "serviceAccount:${google_service_account.ci[each.value[0]].email}"
}
# These project-only APIs manage central infrastructure, not environment jobs.
# Storage is split out below to exclude every trust/state bucket from policy
# changes. GKE gets no Kubernetes object verbs and no getCredentials permission.
resource "google_project_iam_member" "shared" {
  for_each = merge(
    { for role in ["roles/cloudsql.admin", "roles/servicenetworking.networksAdmin", "roles/serviceusage.serviceUsageAdmin"] : "shared_preview:${role}" => ["shared_preview", role] },
    { for role in ["roles/compute.networkAdmin", "roles/compute.securityAdmin", "roles/compute.loadBalancerAdmin", "roles/servicenetworking.networksAdmin", "roles/cloudsql.admin", "roles/dns.admin", "roles/monitoring.editor", "roles/serviceusage.serviceUsageAdmin"] : "shared_production:${role}" => ["shared_production", role] },
  )
  project = var.project_id
  role    = each.value[1]
  member  = "serviceAccount:${google_service_account.ci[each.value[0]].email}"
}
resource "google_project_iam_custom_role" "shared_api" {
  role_id = "ciproxySharedApi"
  title   = "ciproxy central operation and storage metadata"
  permissions = [
    "container.operations.get", "container.operations.list",
    "container.getServerConfig", "storage.buckets.create",
    "cloudkms.keyRings.create", "secretmanager.secrets.create",
  ]
}
resource "google_project_iam_member" "shared_api" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.shared_api.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
}
resource "google_service_account_iam_member" "node_user" {
  for_each           = toset(["shared_production"])
  service_account_id = google_service_account.nodes.name
  role               = "roles/iam.serviceAccountUser"
  member             = "serviceAccount:${google_service_account.ci[each.key].email}"
}
resource "google_project_iam_member" "shared_secret" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = "roles/secretmanager.admin"
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
  condition {
    title = "environment-secret-containers"
    expression = join(" || ", [for env in local.account_environments[each.key] :
      "resource.name.startsWith('projects/${data.google_project.current.number}/secrets/ciproxy-${env}-')"
    ])
  }
}
data "google_project" "current" { project_id = var.project_id }
resource "google_project_iam_member" "shared_kms" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = "roles/cloudkms.admin"
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
  condition {
    title = "environment-key-rings"
    expression = join(" || ", [for env in local.account_environments[each.key] :
      "resource.name == 'projects/${var.project_id}/locations/europe-west1/keyRings/ciproxy-${env}' || resource.name.startsWith('projects/${var.project_id}/locations/europe-west1/keyRings/ciproxy-${env}/')"
    ])
  }
}
resource "google_project_iam_custom_role" "bucket_metadata" {
  role_id     = "ciproxyBucketMetadata"
  title       = "ciproxy environment bucket metadata"
  permissions = ["storage.buckets.get", "storage.buckets.update", "storage.buckets.delete"]
}
resource "google_project_iam_member" "bucket_metadata" {
  for_each = local.shared_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.bucket_metadata.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
  condition {
    title = "public-and-image-buckets-only"
    expression = join(" || ", [for pair in setproduct(local.account_environments[each.key], ["site", "dashboard", "images"]) :
      "resource.name == 'projects/_/buckets/ciproxy-${pair[0]}-${pair[1]}-${var.project_id}'"
    ])
  }
}

resource "google_project_iam_custom_role" "plan_metadata" {
  role_id = "ciproxyPlanMetadata"
  title   = "ciproxy Terraform metadata only"
  permissions = [
    "container.clusters.get", "container.clusters.list",
    "storage.buckets.get", "storage.buckets.getIamPolicy",
    "artifactregistry.repositories.get", "artifactregistry.repositories.getIamPolicy",
    "cloudsql.instances.get", "cloudsql.databases.get", "cloudsql.databases.list", "cloudsql.users.list",
    "monitoring.notificationChannels.get", "monitoring.alertPolicies.get", "monitoring.uptimeCheckConfigs.get",
    "compute.projects.get", "compute.networks.get", "compute.subnetworks.get", "compute.routers.get",
    "compute.globalAddresses.get", "compute.firewalls.get", "compute.backendBuckets.get",
    "compute.healthChecks.get", "compute.backendServices.get", "compute.urlMaps.get",
    "compute.sslCertificates.get", "compute.sslPolicies.get", "compute.targetHttpsProxies.get",
    "compute.targetHttpProxies.get", "compute.globalForwardingRules.get",
    "compute.globalOperations.get", "compute.regionOperations.get", "compute.zoneOperations.get",
    "compute.networkEndpointGroups.get", "compute.urlMaps.listTagBindings", "compute.urlMaps.listEffectiveTags",
    "servicenetworking.services.get",
  ]
}
resource "google_project_iam_member" "plan_metadata" {
  for_each = local.plan_accounts
  project  = var.project_id
  role     = google_project_iam_custom_role.plan_metadata.name
  member   = "serviceAccount:${google_service_account.ci[each.key].email}"
}
