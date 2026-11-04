#!/usr/bin/env bash
# Apply infrastructure only when a local operator deliberately runs this file.
set -euo pipefail

environment="${1:?Usage: deploy/scripts/bootstrap.sh staging|production operator@example.com}"
operator_email="${2:?Supply the uptime alert address.}"
case "${environment}" in
  staging) project_id=internal-498318 ;;
  production) project_id=ciproxy ;;
  *) echo "Unknown environment: ${environment}" >&2; exit 1 ;;
esac

repository_root="$(git rev-parse --show-toplevel)"
scratch="${repository_root}/.context/deploy/${environment}"
mkdir -p "${scratch}"
state_bucket="ciproxy-${environment}-tfstate-${project_id}"

# First authenticate locally with:
# gcloud auth login
# gcloud auth application-default login
# gcloud auth application-default set-quota-project PROJECT_ID
gcloud services enable storage.googleapis.com serviceusage.googleapis.com \
  cloudresourcemanager.googleapis.com --project="${project_id}"

terraform -chdir=deploy/terraform/seed init
terraform -chdir=deploy/terraform/seed apply \
  -state="${scratch}/seed.tfstate" \
  -var="environment=${environment}" -var="project_id=${project_id}"

# Remote state is now available. WIF does not exist yet, so these commands
# still use the operator's Application Default Credentials.
terraform -chdir=deploy/terraform/bootstrap init -reconfigure \
  -backend-config="bucket=${state_bucket}" \
  -backend-config="prefix=${environment}/bootstrap"
terraform -chdir=deploy/terraform/bootstrap apply \
  -var="environment=${environment}" -var="project_id=${project_id}"

python3 - "${operator_email}" "${scratch}/operator.tfvars.json" <<'PY'
import json
import pathlib
import sys
pathlib.Path(sys.argv[2]).write_text(json.dumps({"notification_emails": [sys.argv[1]]}))
PY
terraform -chdir="deploy/environments/${environment}" init -reconfigure \
  -backend-config="bucket=${state_bucket}" \
  -backend-config="prefix=${environment}/foundation"
terraform -chdir="deploy/environments/${environment}" apply \
  -var-file="${scratch}/operator.tfvars.json"

terraform -chdir=deploy/terraform/bootstrap output -json github_environment_variables
terraform -chdir="deploy/environments/${environment}" output -json deployment
terraform -chdir="deploy/environments/${environment}" output dns_name_servers
printf '\nKeep %s/seed.tfstate in a secure backup.\n' "${scratch}"
