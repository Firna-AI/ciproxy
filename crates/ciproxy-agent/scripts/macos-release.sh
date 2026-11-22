#!/usr/bin/env bash
# Apache-2.0. Signs, notarizes, and checks the macOS agent binary.
#
# sign <binary>
#   With every MACOS_* signing input set: Developer ID signature, hardened
#   runtime, the agent entitlements, and Apple notarization.
#   With no MACOS_* signing input set: an ad-hoc signature with the same
#   entitlements. Such a binary is not for release.
#   With only some inputs set: failure, because the setup is incomplete.
#   Prints signed=true or signed=false on stdout.
# verify <binary> <minimum macOS> <signed> <version>
#   Checks the architecture, the minimum macOS, the signature, the
#   entitlements, the notarization when signed is true, and --version.
set -euo pipefail

fail() { printf 'error: %s\n' "$1" >&2; exit 64; }
log() { printf '%s\n' "$1" >&2; }

script_dir="$(cd "$(dirname "$0")" && pwd)"
entitlements="$script_dir/../macos/ciproxy-agent.entitlements"
identifier='dev.ciproxy.agent'
inputs=(
  MACOS_SIGNING_CERTIFICATE_P12_BASE64
  MACOS_SIGNING_CERTIFICATE_PASSWORD
  MACOS_SIGNING_TEAM_ID
  MACOS_NOTARY_API_KEY_P8_BASE64
  MACOS_NOTARY_API_KEY_ID
  MACOS_NOTARY_API_ISSUER_ID
)

signing_mode() {
  local name present=0 missing=()
  for name in "${inputs[@]}"; do
    if [ -n "${!name:-}" ]; then present=$((present + 1)); else missing+=("$name"); fi
  done
  if [ "$present" -eq "${#inputs[@]}" ]; then
    printf 'developer-id'
  elif [ "$present" -eq 0 ]; then
    printf 'ad-hoc'
  else
    fail "macOS signing is set up only in part; missing: ${missing[*]}"
  fi
}

work_dir=''
keychain=''
cleanup() {
  if [ -n "$keychain" ]; then security delete-keychain "$keychain" >/dev/null 2>&1 || true; fi
  if [ -n "$work_dir" ]; then rm -rf "$work_dir"; fi
}

sign_ad_hoc() {
  local binary="$1"
  codesign --force --sign - --options runtime --identifier "$identifier" \
    --entitlements "$entitlements" "$binary"
  log 'NOT FOR RELEASE: the macOS agent has an ad-hoc signature only.'
  log 'NOT FOR RELEASE: no Developer ID signature and no Apple notarization.'
  if [ -n "${GITHUB_ACTIONS:-}" ]; then
    printf '::warning title=macOS agent not for release::%s\n' \
      'The macOS signing secrets are absent. The binary has an ad-hoc signature, no Developer ID signature, and no notarization. Do not publish it.'
  fi
}

import_certificate() {
  local password
  password="$(openssl rand -hex 24)"
  keychain="$work_dir/ciproxy-signing.keychain-db"
  printf '%s' "$MACOS_SIGNING_CERTIFICATE_P12_BASE64" | base64 --decode >"$work_dir/certificate.p12"
  security create-keychain -p "$password" "$keychain"
  security set-keychain-settings -lut 3600 "$keychain"
  security unlock-keychain -p "$password" "$keychain"
  security import "$work_dir/certificate.p12" -k "$keychain" \
    -P "$MACOS_SIGNING_CERTIFICATE_PASSWORD" -T /usr/bin/codesign >/dev/null
  rm -f "$work_dir/certificate.p12"
  security set-key-partition-list -S apple-tool:,apple:,codesign: -s \
    -k "$password" "$keychain" >/dev/null
  # codesign looks up the certificate chain in the search list.
  local existing=()
  while IFS= read -r line; do
    line="${line#"${line%%[![:space:]]*}"}"
    existing+=("${line//\"/}")
  done < <(security list-keychains -d user)
  security list-keychains -d user -s "$keychain" "${existing[@]}"
}

developer_id_hash() {
  local hashes count
  hashes="$(security find-identity -v -p codesigning "$keychain" |
    awk '/"Developer ID Application: / { print $2 }')"
  count="$(printf '%s' "$hashes" | grep -c . || true)"
  [ "$count" -eq 1 ] || fail "the certificate must hold one Developer ID Application identity; found $count"
  printf '%s' "$hashes"
}

notarize() {
  local binary="$1" zip="$work_dir/ciproxy-agent.zip" result="$work_dir/notary.json"
  local status submission
  ditto -c -k --keepParent "$binary" "$zip"
  printf '%s' "$MACOS_NOTARY_API_KEY_P8_BASE64" | base64 --decode >"$work_dir/notary-key.p8"
  local auth=(--key "$work_dir/notary-key.p8" --key-id "$MACOS_NOTARY_API_KEY_ID"
    --issuer "$MACOS_NOTARY_API_ISSUER_ID")
  log 'Submitting the agent to Apple notarization and waiting for the result.'
  xcrun notarytool submit "$zip" "${auth[@]}" --wait --timeout 45m \
    --output-format json >"$result" || true
  status="$(plutil -extract status raw -o - "$result" 2>/dev/null || true)"
  submission="$(plutil -extract id raw -o - "$result" 2>/dev/null || true)"
  log "Notarization submission ${submission:-unknown}: ${status:-no result}"
  if [ "$status" != 'Accepted' ]; then
    if [ -n "$submission" ]; then xcrun notarytool log "$submission" "${auth[@]}" >&2 || true; fi
    fail 'Apple notarization did not accept the agent'
  fi
}

sign_developer_id() {
  local binary="$1" identity
  work_dir="$(mktemp -d)"
  trap cleanup EXIT
  import_certificate
  identity="$(developer_id_hash)"
  codesign --force --sign "$identity" --keychain "$keychain" --timestamp \
    --options runtime --identifier "$identifier" \
    --entitlements "$entitlements" "$binary"
  notarize "$binary"
  cleanup
  trap - EXIT
  log 'The macOS agent has a Developer ID signature and Apple accepted its notarization.'
}

sign_binary() {
  [ "$#" -eq 1 ] || fail 'sign needs the binary path'
  [ -f "$1" ] || fail 'binary path does not exist'
  [ -f "$entitlements" ] || fail 'the entitlements file is missing'
  local mode
  mode="$(signing_mode)"
  case "$mode" in
    developer-id) sign_developer_id "$1"; printf 'signed=true\n' ;;
    ad-hoc) sign_ad_hoc "$1"; printf 'signed=false\n' ;;
  esac
}

check_notarization() {
  local binary="$1" attempt
  # Apple serves a new ticket after a short delay.
  for attempt in 1 2 3 4 5 6; do
    if codesign --verify --strict --check-notarization -R='notarized' "$binary" 2>/dev/null; then
      return 0
    fi
    log "The notarization ticket is not visible yet (attempt $attempt)."
    sleep 20
  done
  fail 'the binary has no notarization ticket'
}

verify_binary() {
  [ "$#" -eq 4 ] || fail 'verify needs binary, minimum macOS, signed, and version'
  local binary="$1" minimum="$2" signed="$3" version="$4" archs minos details granted
  [ -f "$binary" ] || fail 'binary path does not exist'
  archs="$(lipo -archs "$binary")"
  [ "$archs" = 'arm64' ] || fail "binary architectures are '$archs'; release permits arm64 only"
  minos="$(vtool -show-build "$binary" | awk '$1 == "minos" { print $2; exit }')"
  [ "$minos" = "$minimum" ] || fail "binary needs macOS $minos; release expects $minimum"
  codesign --verify --strict --verbose=2 "$binary"
  details="$(codesign --display --verbose=4 "$binary" 2>&1)"
  grep -q "^Identifier=$identifier\$" <<<"$details" || fail "signing identifier is not $identifier"
  grep -q '^CodeDirectory .*flags=.*runtime' <<<"$details" || fail 'hardened runtime is off'
  granted="$(mktemp)"
  codesign --display --entitlements - --xml "$binary" >"$granted" 2>/dev/null
  [ "$(/usr/libexec/PlistBuddy -c 'Print :com.apple.security.virtualization' "$granted" 2>/dev/null)" = 'true' ] ||
    { rm -f "$granted"; fail 'the virtualization entitlement is missing'; }
  rm -f "$granted"
  case "$signed" in
    true)
      grep -q '^Authority=Developer ID Application: ' <<<"$details" || fail 'no Developer ID signature'
      grep -q "^TeamIdentifier=${MACOS_SIGNING_TEAM_ID:?MACOS_SIGNING_TEAM_ID is required}\$" <<<"$details" ||
        fail 'the signature has another team'
      check_notarization "$binary"
      ;;
    false)
      grep -q '^Signature=adhoc$' <<<"$details" || fail 'expected an ad-hoc signature'
      log 'NOT FOR RELEASE: verified an ad-hoc signature only.'
      ;;
    *) fail 'signed must be true or false' ;;
  esac
  "$binary" --version | grep -q -- " $version\$" || fail "binary --version does not report $version"
  printf 'arch=%s\nminos=%s\nsigned=%s\n' "$archs" "$minos" "$signed"
}

[ "$#" -gt 0 ] || fail 'use sign or verify'
command="$1"
shift
case "$command" in
  sign) sign_binary "$@" ;;
  verify) verify_binary "$@" ;;
  *) fail 'use sign or verify' ;;
esac
