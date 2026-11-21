# Prelaunch may create preview/core resources, but cannot install IAM on the
# shared network or registry. Fresh protected production state refuses adoption.
resource "google_project_iam_custom_role" "prelaunch" {
  role_id = "ciproxyPrelaunchInfrastructure"
  title   = "ciproxy prelaunch infrastructure without shared IAM setters"
  permissions = [
    "compute.networks.create", "compute.networks.delete", "compute.networks.get", "compute.networks.update", "compute.networks.updatePolicy", "compute.networks.switchToCustomMode", "compute.networks.use", "compute.networks.addPeering", "compute.networks.removePeering", "compute.networks.updatePeering",
    "compute.subnetworks.create", "compute.subnetworks.delete", "compute.subnetworks.get", "compute.subnetworks.update", "compute.subnetworks.expandIpCidrRange", "compute.subnetworks.use",
    "compute.routers.create", "compute.routers.delete", "compute.routers.get", "compute.routers.update", "compute.routers.use",
    "compute.firewalls.create", "compute.firewalls.delete", "compute.firewalls.get", "compute.firewalls.update",
    "compute.globalAddresses.create", "compute.globalAddresses.createInternal", "compute.globalAddresses.delete", "compute.globalAddresses.deleteInternal", "compute.globalAddresses.get", "compute.globalAddresses.setLabels", "compute.globalAddresses.use",
    "compute.backendBuckets.create", "compute.backendBuckets.delete", "compute.backendBuckets.get", "compute.backendBuckets.update", "compute.backendBuckets.use",
    "compute.backendServices.create", "compute.backendServices.delete", "compute.backendServices.get", "compute.backendServices.update", "compute.backendServices.use",
    "compute.healthChecks.create", "compute.healthChecks.delete", "compute.healthChecks.get", "compute.healthChecks.update", "compute.healthChecks.useReadOnly",
    "compute.urlMaps.create", "compute.urlMaps.delete", "compute.urlMaps.get", "compute.urlMaps.update", "compute.urlMaps.use", "compute.urlMaps.validate", "compute.urlMaps.invalidateCache",
    "compute.sslCertificates.create", "compute.sslCertificates.delete", "compute.sslCertificates.get",
    "compute.sslPolicies.create", "compute.sslPolicies.delete", "compute.sslPolicies.get", "compute.sslPolicies.update", "compute.sslPolicies.use",
    "compute.targetHttpProxies.create", "compute.targetHttpProxies.delete", "compute.targetHttpProxies.get", "compute.targetHttpProxies.setUrlMap", "compute.targetHttpProxies.use",
    "compute.targetHttpsProxies.create", "compute.targetHttpsProxies.delete", "compute.targetHttpsProxies.get", "compute.targetHttpsProxies.setUrlMap", "compute.targetHttpsProxies.setSslCertificates", "compute.targetHttpsProxies.setSslPolicy", "compute.targetHttpsProxies.use",
    "compute.globalForwardingRules.create", "compute.globalForwardingRules.delete", "compute.globalForwardingRules.get", "compute.globalForwardingRules.setLabels", "compute.globalForwardingRules.setTarget", "compute.globalForwardingRules.update",
    "compute.globalOperations.get",
    "compute.regionOperations.get",
    "compute.zoneOperations.get",
    "compute.projects.get",
    "compute.regions.get",
    "compute.zones.get",
    "compute.networkEndpointGroups.get", "compute.networkEndpointGroups.use",
    "dns.managedZones.create", "dns.managedZones.delete", "dns.managedZones.get", "dns.managedZones.list", "dns.managedZones.update",
    "dns.resourceRecordSets.create", "dns.resourceRecordSets.delete", "dns.resourceRecordSets.list", "dns.resourceRecordSets.update",
    "dns.changes.create", "dns.changes.get", "dns.changes.list",
    "monitoring.notificationChannels.create", "monitoring.notificationChannels.delete", "monitoring.notificationChannels.get", "monitoring.notificationChannels.list", "monitoring.notificationChannels.update",
    "monitoring.uptimeCheckConfigs.create", "monitoring.uptimeCheckConfigs.delete", "monitoring.uptimeCheckConfigs.get", "monitoring.uptimeCheckConfigs.list", "monitoring.uptimeCheckConfigs.update",
    "monitoring.alertPolicies.create", "monitoring.alertPolicies.delete", "monitoring.alertPolicies.get", "monitoring.alertPolicies.list", "monitoring.alertPolicies.update",
  ]
}
resource "google_project_iam_member" "prelaunch" {
  project = var.project_id
  role    = google_project_iam_custom_role.prelaunch.name
  member  = "serviceAccount:${google_service_account.ci["shared_preview"].email}"
}
