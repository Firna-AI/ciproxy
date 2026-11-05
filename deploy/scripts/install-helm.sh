#!/usr/bin/env bash
set -euo pipefail

version=v4.3.1
# Checksums: https://github.com/helm/helm/releases/tag/v4.3.1
# Each archive's .sha256sum at https://get.helm.sh/ has the same digest.
system="$(uname -s)"
machine="$(uname -m)"
case "${system}/${machine}" in
  Linux/x86_64|Linux/amd64)
    platform=linux-amd64
    checksum=0403a88ae20913d11b10a2467b50087721fea7581736da6b9d3992f5357fe54b
    ;;
  Linux/aarch64|Linux/arm64)
    platform=linux-arm64
    checksum=8cf8f66ecbc252ac8f7f06cca68aba88508ddfde3a6a34feb1678ad1694b4f88
    ;;
  Darwin/arm64|Darwin/aarch64)
    platform=darwin-arm64
    checksum=6b4e61c8cd27d99e5a801e63ff25dd5c70b36875c2cd659d3e9ce6b76fc23198
    ;;
  Darwin/x86_64|Darwin/amd64)
    platform=darwin-amd64
    checksum=34bec4eba761ba5c16d1965ee6b8f5b0abe5993974a802274e2a4f0418893d89
    ;;
  *)
    printf 'Unsupported Helm platform: %s/%s. Use Linux or macOS on amd64 or arm64.\n' \
      "${system}" "${machine}" >&2
    exit 1
    ;;
esac

if command -v sha256sum >/dev/null 2>&1; then
  checker=(sha256sum)
else
  checker=(shasum -a 256)
fi
archive="helm-${version}-${platform}.tar.gz"
scratch="${RUNNER_TEMP:-${PWD}/.context/tools}/ciproxy-helm"
mkdir -p "${scratch}"
curl --fail --silent --show-error --location \
  "https://get.helm.sh/${archive}" --output "${scratch}/${archive}"
if ! (
  cd "${scratch}"
  printf '%s  %s\n' "${checksum}" "${archive}" | "${checker[@]}" -c
); then
  printf 'SHA-256 verification failed for %s. Helm was not installed.\n' "${archive}" >&2
  exit 1
fi
tar -xzf "${scratch}/${archive}" -C "${scratch}" "${platform}/helm"
install -m 0755 "${scratch}/${platform}/helm" "${scratch}/helm"
if [[ -n "${GITHUB_PATH:-}" ]]; then
  printf '%s\n' "${scratch}" >> "${GITHUB_PATH}"
else
  printf 'Helm installed: %s/helm\n' "${scratch}"
fi
