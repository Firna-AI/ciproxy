#!/usr/bin/env bash
# Deliberately changes live signing configuration. Never run during validation.
set -euo pipefail

environment="${1:?Usage: register-cdn-key.sh staging|production KEY_FILE}"
key_file="${2:?Supply a file with a user-provided 128-bit base64url key.}"
case "${environment}" in
  staging) project_id=internal-498318 ;;
  production) project_id=ciproxy ;;
  *) echo "Unknown environment: ${environment}" >&2; exit 1 ;;
esac
prefix="ciproxy-${environment}"

# A numbered version and its public CDN key name are one selector. Capture
# only version metadata. Neither CI nor Terraform receives the key value.
secret_version="$(gcloud secrets versions add "${prefix}-cdn-signing-key" \
  --data-file="${key_file}" --project="${project_id}" --format='value(name)')"
version_number="${secret_version##*/}"
key_name="${prefix}-images-v${version_number}"

# Register before granting cache-fill IAM. GCP creates the cache-fill service
# account when the first key is registered. Publish the selector only after
# all operations succeed, so a failed registration cannot activate it.
gcloud compute backend-buckets add-signed-url-key "${prefix}-images" \
  --key-name="${key_name}" --key-file="${key_file}" --project="${project_id}"
project_number="$(gcloud projects describe "${project_id}" --format='value(projectNumber)')"
gcloud storage buckets add-iam-policy-binding \
  "gs://${prefix}-images-${project_id}" \
  --member="serviceAccount:service-${project_number}@cloud-cdn-fill.iam.gserviceaccount.com" \
  --role=roles/storage.objectViewer
printf 'Registered CDN key %s. Configure the runtime signer with this name.\n' "${key_name}"
printf 'GitHub environment variable CDN_SIGNING_KEY_NAME=%s\n' "${key_name}"
