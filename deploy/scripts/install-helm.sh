#!/usr/bin/env bash
set -euo pipefail

# The deployment runner is Ubuntu x86_64. No package repository is needed.
version=v4.3.1
scratch="${RUNNER_TEMP:-.context/tools}/ciproxy-helm"
mkdir -p "${scratch}"
curl --fail --silent --show-error --location \
  "https://get.helm.sh/helm-${version}-linux-amd64.tar.gz" \
  --output "${scratch}/helm-${version}-linux-amd64.tar.gz"
curl --fail --silent --show-error --location \
  "https://get.helm.sh/helm-${version}-linux-amd64.tar.gz.sha256sum" \
  --output "${scratch}/helm-${version}-linux-amd64.tar.gz.sha256sum"
(
  cd "${scratch}"
  sha256sum --check "helm-${version}-linux-amd64.tar.gz.sha256sum"
)
tar -xzf "${scratch}/helm-${version}-linux-amd64.tar.gz" -C "${scratch}" linux-amd64/helm
install -m 0755 "${scratch}/linux-amd64/helm" "${scratch}/helm"
if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "${scratch}" >> "${GITHUB_PATH}"
else
  printf 'Helm installed: %s/helm\n' "${scratch}"
fi
