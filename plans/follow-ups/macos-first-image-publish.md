# Publish and pin the first macOS release image

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from `calummoore/h4-publish-and-xcode-fetch-gomg`,
  the post-merge task of the deleted plan `plans/macos-image-hosting.md`.
- **Location:** `image/macos/publish.sh`, `image/macos/README.md`
  ("Publish"), `deploy/runtime.md` ("macOS images"), `deploy/setup.md`.
- **Reason:** The publish runs on the build Mac with its R2 keys, after the
  scripts and the control-plane reader are on `main`.

- **Terms:**
  - **Build Mac:** the test Mac mini that builds the macOS image. Its R2
    keys live in `~/ws-i/secrets/r2-publish.env`.
  - **Standard variant:** the macOS image without the extra simulator
    runtimes. Its image ID is `STANDARD_IMAGE_ID` in
    `image/macos/pins.env`.
- **Background:**
  - The control plane offers one macOS release for each environment. It
    reads the release only when `MACOS_IMAGE_URL_PREFIX`,
    `MACOS_IMAGE_MANIFEST_PATH`, and `R2_IMAGES_KEY` are all set, as
    `deploy/runtime.md` says under "macOS images".
  - Decision, 23 November 2026, by the parent workspace H under Cal's
    authority: both environments offer the standard variant for now,
    because only the standard image fits every Mac today. The full
    variant waits for `plans/follow-ups/macos-image-variant-per-mac.md`.
  - The standard image
    `macos26-arm64-20261117.0516-runner-2.338.2-xcode-26.6-standard-v1`
    has a `macos.tar` of 47,559,669,760 bytes.
  - Status, 23 November 2026: the standard v2 image (100 GB disk,
    `…-xcode-26.6-standard-v2`, `macos.tar` 46,190,479,360 bytes) is
    published. Both GitHub environments pin it. Both Secret Manager
    secrets have their read key. Main preview offers it, and the test Mac
    downloaded it from R2 and ran a job with Xcode (run 45212978247 in
    `firna-test-org/ciproxy-macos-spike`). Only steps 4 (deploy
    production) and the production half of "Done when" remain.
  - The two environment read tokens, `ciproxy-main-preview-macos-images-read`
    and `ciproxy-production-macos-images-read`, go into the Secret Manager
    secrets `ciproxy-main-preview-r2-images` and
    `ciproxy-production-r2-images` of project `ciproxy`. Each version holds
    `{"access_key_id": "<token ID>", "secret_access_key": "<SHA-256 hex of
    the token value>"}`. Add a version only after the Terraform data apply
    creates the secret.
- **Task:**
  1. On the build Mac, publish the standard image from
     `/Volumes/cals-sandisk/ciproxy-build-standard/artifacts` with
     `publish.sh`, as the README section "Publish" says.
  2. Add the read key version to the Secret Manager secret of each
     environment.
  3. In the GitHub environments `main-preview` and `production`, set
     `MACOS_IMAGE_URL_PREFIX` to
     `https://785233830ca9dfa2eb7c3561dd52f675.r2.cloudflarestorage.com/ciproxy-macos-images`,
     and `MACOS_IMAGE_MANIFEST_PATH` to the value that `publish.sh` prints.
     Both environments pin the standard image.
  4. Deploy `main-preview`, then `production`.
  5. Watch a Mac with the agent download and publish the image.
- **Impact if no action:** No customer Mac gets a macOS image, so no macOS
  job runs on a customer Mac.
- **Needs:** The build Mac with `r2-publish.env`, the two read tokens, and
  rights to set GitHub environment variables and deploy. The parent
  workspace H runs the Mac steps.
- **Done when:** A Mac in `main-preview` reports the standard image as
  ready, and `production` pins the same release.
