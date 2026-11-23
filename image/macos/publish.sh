#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0
# Publish a packaged macOS image to releases/<image_id>/ of the private R2
# bucket ciproxy-macos-images. Run on the build Mac after package.sh, with
# R2_ACCOUNT_ID, R2_PUBLISH_ACCESS_KEY_ID, and R2_PUBLISH_SECRET_ACCESS_KEY set.
# A release folder is immutable, and manifest.json goes up last, so a folder
# with a manifest is complete.
set -euo pipefail

definition=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$definition/r2.sh"
usage="Usage: image/macos/publish.sh [--test-image] ARTIFACT_DIR"
bucket=ciproxy-macos-images
# The first AWS CLI v2 release with --if-none-match on s3api put-object.
aws_min_version=2.17.34
small_files=(downloads.json SHA256SUMS measurement.json software-report.json
    upstream-software.json size-report.txt)

allow_test_image=false
if [[ ${1:-} == --test-image ]]; then
    allow_test_image=true
    shift
fi
[[ $# == 1 ]] || { echo "$usage" >&2; exit 2; }
artifacts=$(cd "$1" && pwd)
cd "$artifacts"

fail() {
    echo "$1" >&2
    exit 1
}

publish_aws() {
    r2_aws R2_PUBLISH_ACCESS_KEY_ID R2_PUBLISH_SECRET_ACCESS_KEY "$@"
}

r2_require R2_PUBLISH_ACCESS_KEY_ID R2_PUBLISH_SECRET_ACCESS_KEY
r2_require_aws "$aws_min_version"
for file in macos.tar "${small_files[@]}" manifest.json; do
    [[ -f $file ]] || fail "$artifacts has no $file. Run package.sh first."
done

# Prints the image ID, the variant, and the archive size. write-manifest.py
# writes the variant: full, standard, or test. An older build has none.
read_release() {
    python3 -I - <<'PY'
import json, re, sys
manifest = json.load(open("manifest.json"))
downloads = json.load(open("downloads.json"))
image_id = manifest.get("image_id", "")
if not re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._+-]*", image_id):
    sys.exit(f"manifest.json has no valid image_id: {image_id!r}")
test_image = manifest.get("test_image") is True
variant = manifest.get("variant")
if variant is None:
    variant = "test" if test_image else "full"
    print(f"Warning: manifest.json has no variant, so it is an older build. "
          f"It counts as {variant}.", file=sys.stderr)
if variant not in ("full", "standard", "test"):
    sys.exit(f"manifest.json has an unknown variant: {variant!r}")
if (variant == "test") != test_image:
    sys.exit(f"manifest.json says variant {variant} and test_image "
             f"{str(test_image).lower()}. Only a test variant is a test image.")
print(image_id, variant, int(downloads["macos"]["size_bytes"]))
PY
}

release=$(read_release)
read -r image_id variant size_bytes <<<"$release"
if [[ $variant == test && $allow_test_image == false ]]; then
    fail "manifest.json marks $image_id as a test image. Give --test-image to publish it."
fi
echo "Publishing $image_id, variant $variant."

shasum -a 256 -c SHA256SUMS

prefix=releases/$image_id
if head_error=$(publish_aws s3api head-object --bucket "$bucket" \
    --key "$prefix/manifest.json" 2>&1 >/dev/null); then
    fail "s3://$bucket/$prefix/manifest.json exists. A release is immutable: change the image ID."
fi
case $head_error in
*"(404)"* | *"Not Found"*) ;;
*) fail "Could not check s3://$bucket/$prefix/manifest.json: $head_error" ;;
esac

echo "Uploading macos.tar in parts of $R2_PART_BYTES bytes."
publish_aws s3 cp macos.tar "s3://$bucket/$prefix/macos.tar"
for file in "${small_files[@]}"; do
    publish_aws s3 cp "$file" "s3://$bucket/$prefix/$file" --only-show-errors
done

# The size check comes before the manifest, so a folder with a manifest also
# has the full archive.
uploaded_bytes=$(publish_aws s3api head-object --bucket "$bucket" \
    --key "$prefix/macos.tar" --query ContentLength --output text)
if [[ $uploaded_bytes != "$size_bytes" ]]; then
    fail "The uploaded macos.tar has $uploaded_bytes bytes, downloads.json says $size_bytes. The release has no manifest."
fi

# If-None-Match: * lets only one of two publishes of a release write it.
if ! publish_aws s3api put-object --bucket "$bucket" --key "$prefix/manifest.json" \
    --body manifest.json --content-type application/json --if-none-match '*' >/dev/null; then
    fail "The upload of manifest.json failed. When another publish wrote it first, the release exists."
fi
echo "Published $image_id, variant $variant."
echo "MACOS_IMAGE_MANIFEST_PATH=$prefix/manifest.json"
