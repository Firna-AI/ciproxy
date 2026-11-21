#!/usr/bin/env bash
set -euo pipefail
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
if [[ ${1:-} == build ]]; then
  curl -fsSL --retry 3 \
    https://github.com/moby/buildkit/releases/download/v0.33.0/buildkit-v0.33.0.linux-amd64.tar.gz \
    -o "$temporary/buildkit.tar.gz"
  printf '%s  %s\n' b6242896d343100808dcbe37565caf381e0a444a6a83d7255926bb1519248ead \
    "$temporary/buildkit.tar.gz" | sha256sum --check
  tar -xzf "$temporary/buildkit.tar.gz" -C "$temporary"
  sudo install -m 755 "$temporary/bin/buildctl" "$temporary/bin/buildkitd" \
    "$temporary/bin/buildkit-runc" /usr/local/bin/
else
  curl -fsSL --retry 3 \
    https://github.com/google/go-containerregistry/releases/download/v0.22.1/go-containerregistry_Linux_x86_64.tar.gz \
    -o "$temporary/crane.tar.gz"
  printf '%s  %s\n' 0ab7a1d6932a213aed964ce97666c3077fe691c8606413674a8b3e0b9ec4cda0 \
    "$temporary/crane.tar.gz" | sha256sum --check
  tar -xzf "$temporary/crane.tar.gz" -C "$temporary" crane
  sudo install -m 755 "$temporary/crane" /usr/local/bin/
fi
