#!/usr/bin/env bash
# Apache-2.0. Archive contract from futex-ai/cli-release at 30513bd1a6e1.
set -euo pipefail

fail() { printf 'error: %s\n' "$1" >&2; exit 64; }

validate_name() {
  case "$1" in
    ''|*[!A-Za-z0-9._-]*) fail "$2 contains unsupported characters" ;;
  esac
}

package_cli() {
  [ "$#" -eq 5 ] || fail 'package needs binary, version, target, binary path, and output directory'
  local binary="$1" version="$2" target="$3" binary_path="$4" dist_dir="$5"
  validate_name "$binary" binary
  validate_name "$version" version
  validate_name "$target" target
  case "$version" in v*) fail 'version must not include a leading v' ;; esac
  [ -f "$binary_path" ] || fail 'binary path does not exist'
  local artifact="${binary}-${version}-${target}" staging
  staging="$(mktemp -d)"
  trap 'rm -rf "$staging"' EXIT
  mkdir -p "$staging/$artifact" "$dist_dir"
  cp "$binary_path" "$staging/$artifact/$binary"
  chmod 755 "$staging/$artifact/$binary"
  if [ "$binary" = ciproxy-agent ]; then
    cp crates/ciproxy-agent/LICENSE "$staging/$artifact/LICENSE"
  fi
  tar -C "$staging" -czf "$dist_dir/$artifact.tar.gz" "$artifact"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$dist_dir/$artifact.tar.gz" | awk '{print $1}' >"$dist_dir/$artifact.tar.gz.sha256"
  else
    shasum -a 256 "$dist_dir/$artifact.tar.gz" | awk '{print $1}' >"$dist_dir/$artifact.tar.gz.sha256"
  fi
  printf 'asset_name=%s.tar.gz\nasset_path=%s/%s.tar.gz\n' "$artifact" "$dist_dir" "$artifact"
  printf 'checksum_name=%s.tar.gz.sha256\nchecksum_path=%s/%s.tar.gz.sha256\n' "$artifact" "$dist_dir" "$artifact"
  rm -rf "$staging"
  trap - EXIT
}

resolve_tag() {
  [ "$#" -eq 2 ] || fail 'resolve-agent-tag needs a tag and a manifest path'
  local tag="${1#refs/tags/}" manifest="$2" version current
  case "$tag" in
    ciproxy-agent-v*) version="${tag#ciproxy-agent-v}" ;;
    *) fail 'tag must start with ciproxy-agent-v' ;;
  esac
  validate_name "$version" version
  current="$(cargo metadata --no-deps --format-version=1 --manifest-path "$manifest" |
    jq -r '.packages[] | select(.name == "ciproxy-agent") | .version')"
  [ "$version" = "$current" ] || fail 'tag version does not match the agent manifest'
  printf 'version=%s\ntag=%s\n' "$version" "$tag"
}

verify_glibc() {
  [ "$#" -eq 2 ] || fail 'verify-glibc needs a binary path and the maximum glibc version'
  local binary="$1" maximum="$2" actual newest
  actual="$(LC_ALL=C readelf --version-info --wide "$binary" |
    sed -n 's/.*Name: GLIBC_\([0-9.]*\) .*/\1/p' | sort -Vu | tail -n 1)"
  [ -n "$actual" ] || fail 'no glibc version was found in the GNU release binary'
  newest="$(printf '%s\n%s\n' "$actual" "$maximum" | sort -V | tail -n 1)"
  [ "$newest" = "$maximum" ] || fail "binary needs glibc $actual; release permits $maximum"
  printf 'glibc=%s\nmaximum_glibc=%s\n' "$actual" "$maximum"
}

[ "$#" -gt 0 ] || fail 'use package, resolve-agent-tag, or verify-glibc'
command="$1"
shift
case "$command" in
  package) package_cli "$@" ;;
  resolve-agent-tag) resolve_tag "$@" ;;
  verify-glibc) verify_glibc "$@" ;;
  *) fail 'use package, resolve-agent-tag, or verify-glibc' ;;
esac
