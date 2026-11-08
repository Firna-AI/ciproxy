#!/usr/bin/env bash
# Run locally after registry authentication. This command writes an image.
set -euo pipefail

environment="${1:?Usage: mirror-release-image.sh main-preview|production [source|published]}"
mode="${2:-source}"
case "${environment}" in
  main-preview) project_id=ciproxy ;;
  production) project_id=ciproxy ;;
  *) echo "Unknown environment: ${environment}" >&2; exit 1 ;;
esac
revision=53a1dd342622
full_revision=53a1dd3426226a9d25defb4a899d3c9a430a3941
source_image="ghcr.io/futex-ai/cli-release-server:sha-${revision}"
repository="us-central1-docker.pkg.dev/${project_id}/ciproxy-shared-runtime/cli-release-server"
destination="${repository}:upstream-${revision}"

# A local GitHub login needs access to the upstream repository for source,
# or read:packages for a published image. The runtime's public-read-only
# release token is not an upstream source or registry credential.
gcloud auth configure-docker us-central1-docker.pkg.dev --quiet
case "${mode}" in
  source)
    # This path does not assume that upstream CI published an image.
    scratch="$(git rev-parse --show-toplevel)/.context/deploy/release-source/${full_revision}"
    mkdir -p "${scratch}/source"
    gh api "repos/futex-ai/cli-release/tarball/${full_revision}" > "${scratch}/source.tar.gz"
    tar -xzf "${scratch}/source.tar.gz" -C "${scratch}/source" --strip-components=1
    docker build --platform=linux/amd64 --target cli-release-server-runtime \
      --build-arg="CLI_RELEASE_BUILD_COMMIT=${full_revision}" \
      --tag="${destination}" "${scratch}/source"
    docker run --rm --platform=linux/amd64 "${destination}" --help
    ;;
  published)
    docker pull --platform=linux/amd64 "${source_image}"
    docker tag "${source_image}" "${destination}"
    ;;
  *) echo "Unknown image mode: ${mode}" >&2; exit 1 ;;
esac
docker push "${destination}"
digest="$(gcloud artifacts docker images describe "${destination}" \
  --project="${project_id}" --format='value(image_summary.digest)')"
printf 'GitHub environment variable RELEASE_SERVER_IMAGE_DIGEST=%s\n' "${digest}"
