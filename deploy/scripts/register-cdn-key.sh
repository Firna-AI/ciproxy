#!/usr/bin/env bash
# Deliberately changes live signing configuration. Never run during validation.
set -euo pipefail

environment="${1:?Usage: register-cdn-key.sh staging|production KEY_NAME KEY_FILE}"
key_name="${2:?Supply a new signing key name.}"
key_file="${3:?Supply a file with a user-provided 128-bit base64url key.}"
case "${environment}" in
  staging) project_id=internal-498318 ;;
  production) project_id=ciproxy ;;
  *) echo "Unknown environment: ${environment}" >&2; exit 1 ;;
esac
prefix="ciproxy-${environment}"

# Register before granting cache-fill IAM. GCP creates the cache-fill service
# account when the first key is registered. No secret goes through Terraform.
gcloud compute backend-buckets add-signed-url-key "${prefix}-images" \
  --key-name="${key_name}" --key-file="${key_file}" --project="${project_id}"
project_number="$(gcloud projects describe "${project_id}" --format='value(projectNumber)')"
gcloud storage buckets add-iam-policy-binding \
  "gs://${prefix}-images-${project_id}" \
  --member="serviceAccount:service-${project_number}@cloud-cdn-fill.iam.gserviceaccount.com" \
  --role=roles/storage.objectViewer
gcloud secrets versions add "${prefix}-cdn-signing-key" \
  --data-file="${key_file}" --project="${project_id}"
printf 'Registered CDN key %s. Configure the runtime signer with this name.\n' "${key_name}"
printf 'GitHub environment variable CDN_SIGNING_KEY_NAME=%s\n' "${key_name}"
