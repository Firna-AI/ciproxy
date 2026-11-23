# macOS image hosting and delivery

Status: Active. On branch
`calummoore/macos-h-macos-image-hosting-and-delivery-6ebi`. The work lands
in four pull requests, one for each sub-workspace of workstream H, as
"Pull requests" below says. The pull request that completes the work
deletes this plan, as the `plans` skill says.

This is workstream H of the macOS work. The parent plan is
`plans/macos-own-machines.md` on branch `calummoore/support-mac-os-kq1u`.
A Mac runs macOS jobs in VMs that boot the macOS image. Today a person
loads a test image by hand with `ciproxy-agent import-image`. Two gaps stop
a customer Mac from getting an image:

- Nothing publishes the `macos.tar` that the image build of workstream I
  writes (draft PR #113).
- The control plane offers one image only. `CdnReleases` in
  `crates/ciproxy-control-plane/src/image_release.rs` reads one manifest
  and sets `Platform::LinuxX64`.

The agent on a Mac already downloads, checks, and publishes an image when
the control plane sends it a `FetchImage` command with a macOS
`ImageDownload`. `ensure_image` in
`crates/ciproxy-control-plane/src/image_delivery.rs` already sends an image
only to machines of the same platform.

User decisions:

- 21 November 2026: ciproxy builds and serves a ready macOS image with
  Xcode. See the parent plan.
- Decision 3 of the parent workspace: Cloudflare R2 hosts the macOS
  images. The Linux image stays in its GCS bucket behind Cloud CDN.
- Decision 2 of the parent workspace: the build downloads each Xcode `.xip`
  once and keeps a private copy in a private R2 bucket. GitHub does the
  same with `XCODE_INSTALL_STORAGE_URL` and a signed token.
- 23 November 2026, through the parent workspace: Cal chose the
  recommended option of each decision below: 1 B, 2 D, 3 A, 4 B, 5 A, and
  6 A. He also asked for Milestone 2 as its own pull request.

Design calls:

- 23 November 2026, my call: the control plane signs the presigned URL
  with its own code, not with the `rusty-s3` crate (decision 1 E).
  `rusty-s3` 0.12 needs `jiff` 1.0 beside the locked `jiff` 0.2, and a
  crypto backend chosen by feature flags. Own code reuses `hmac` and
  `sha2`, which the control plane already uses, and a test checks it
  against the published AWS example.
- 23 November 2026, my call: one GitHub variable,
  `MACOS_IMAGE_URL_PREFIX`, holds the R2 endpoint and the bucket in path
  style, as `IMAGE_URL_PREFIX` does for Linux. So the operator sets no
  separate account and bucket variables.
- 23 November 2026, my call: the resume logic of decision 2 lives in a
  module of the agent that builds on every platform, so that the Linux
  tests run it. The macOS image store calls it.

Out of scope: a macOS image build in CI, the Boat snapshot, and custom
images.

Contract owners:

- [Agent images](../docs/protocol/agent-images.md), section "The macOS
  image".
- [Runtime](../deploy/runtime.md), the control-plane settings.
- The macOS image README, `image/macos/README.md`, after PR #113 merges.

## How the Linux image works today

- A person builds the image on a Linux build machine.
  `image/publish.sh ARTIFACT_DIR gs://BUCKET/PREFIX` checks `SHA256SUMS`
  and uploads the artifacts to the private image bucket of an environment.
- Each environment pins one release with the GitHub environment variable
  `JOB_IMAGE_MANIFEST_PATH`, for example `<release>/manifest.json`.
- At start, the control plane reads that `manifest.json` and the sibling
  `downloads.json` through Cloud CDN. It signs each read with the CDN key.
  A failed read stops the start.
- `manifest.json` gives `image_id` and `root_disk_bytes`.
  `downloads.json` gives the file, SHA-256, and byte size of the bundle.
- For each `FetchImage`, the control plane signs a fresh Cloud CDN URL that
  expires after 2 hours.
- When the agent reports the result of a `FetchImage`, the control plane
  forgets the request. The next reconciliation sends a new `FetchImage`
  with a fresh URL to a machine that still lacks the image.

## Design

### Bucket layout

One private bucket, `ciproxy-macos-images`, serves both environments. My
call, 23 November 2026: a 30 to 60 GB upload happens once, and each
environment still pins its own release.

```text
ciproxy-macos-images/
  releases/<image_id>/macos.tar
  releases/<image_id>/downloads.json
  releases/<image_id>/SHA256SUMS
  releases/<image_id>/measurement.json
  releases/<image_id>/software-report.json
  releases/<image_id>/upstream-software.json
  releases/<image_id>/size-report.txt
  releases/<image_id>/manifest.json
```

- `<image_id>` is the `image_id` of `manifest.json`, for example
  `macos26-arm64-20261117.0516-runner-2.338.2-xcode-26.6-v1`.
- A release folder is immutable. Publish refuses an `<image_id>` that
  already has a `manifest.json`.
- Publish uploads `manifest.json` last. So a release folder with a manifest
  is complete.
- `downloads.json` has the form that `image/macos/package.sh` writes:
  `{"macos": {"file", "sha256", "size_bytes"}}`.

### Publish

`image/macos/publish.sh [--test-image] ARTIFACT_DIR` runs on the build Mac
after `package.sh`. It reads `R2_ACCOUNT_ID` and the publish token from
environment variables, and it never prints them. The endpoint comes from
`R2_ACCOUNT_ID`. The script fixes the bucket name `ciproxy-macos-images`.

1. Read `image_id` from `manifest.json`. Refuse a test image unless
   `--test-image` is given.
2. Check `SHA256SUMS` in the artifact folder.
3. Stop when `releases/<image_id>/manifest.json` exists.
4. Upload `macos.tar` as a multipart upload with 128 MiB parts. A 60 GB
   file then needs about 450 parts, under the S3 limit of 10,000.
5. Upload the small files.
6. Read back the byte size of `macos.tar` with a HEAD request, and compare
   it with `downloads.json`. This check comes before the manifest, so a
   release folder with a manifest always holds the full archive.
7. Upload `manifest.json` with `If-None-Match: *`, so two publishes of one
   release cannot both write it.
8. Print the value for `MACOS_IMAGE_MANIFEST_PATH`:
   `releases/<image_id>/manifest.json`.

The script uses the AWS CLI v2 with `--endpoint-url`. It sets
`AWS_REQUEST_CHECKSUM_CALCULATION=when_required` and
`AWS_RESPONSE_CHECKSUM_VALIDATION=when_required`, which Cloudflare advises
for R2. `publish.sh` needs AWS CLI 2.17.34 or later, the first release with
conditional writes for `PutObject`. The shared helper `image/macos/r2.sh`
gives the keys only to the environment of each `aws` process.

### Offer one image for each platform

- The control plane holds a list of releases, at most one for each
  platform. `MachineRuntime::image` becomes that list.
- `ensure_image` offers each machine the release of its own platform. A
  Mac gets the macOS release and a Linux machine the Linux release. A
  machine of a platform without a release gets nothing.
- The Linux release keeps its settings, its CDN reads, and its failure at
  start.
- The macOS release is optional. When `MACOS_IMAGE_MANIFEST_PATH` is empty,
  the control plane offers no macOS image and logs that once. Linux jobs
  are not affected.
- When a macOS manifest read fails, the control plane logs a warning and
  starts without a macOS image. It does not stop Linux work. My call, 23
  November 2026: it does not retry. The next pod start reads again, because
  a release changes only with a deploy under the recommended pin.
- A second reader, `R2Releases`, reads `manifest.json` and `downloads.json`
  from R2 with short presigned URLs. It sets `Platform::MacosArm64`, and
  takes the artifact from the `macos` key.
- Each release has its own URL signer: the CDN key for Linux, and the R2
  signer for macOS. Milestone 2 adds the per-release signer.
- The Linux `image_disk_bytes` setting keeps the Linux root size only. The
  macOS root size travels in the macOS `ImageDownload`.

### Authenticated download

Recommended in decision 1: the bucket stays private. The control plane
signs an S3 presigned GET URL (AWS Signature Version 4) for each
`FetchImage`, as it signs a Cloud CDN URL for Linux. The URL names one
object and one operation, and it expires. The agent checks the byte size
and the SHA-256 that the control plane sends. The signing key is a
read-only R2 token, scoped to the image bucket. The control plane
implements the signature itself, tested against the published AWS example.

### Size, time, and cost

- The release image with one Xcode is likely 30 to 60 GB as `macos.tar`.
- At 100 Mbit/s, 60 GB takes about 80 minutes. At 1 Gbit/s, it takes about
  8 minutes.
- A Mac needs free space for the archive and the extracted sparse disk at
  the same time.
- R2 has no egress fee. Storage costs about USD 0.015 per GB and month, so
  one 60 GB release costs about USD 0.90 per month.

### Resumable downloads

Today the agent on a Mac downloads in one stream. A broken connection
starts the 60 GB download again from zero. Decision 2 chooses the resume
behaviour.

### Private Xcode archives

A second private bucket, `ciproxy-xcode`, holds each Xcode `.xip` that
Apple serves after an Apple Account sign-in. My call, 23 November 2026: a
separate bucket, so that no image token can read Xcode archives and no
public setting of one bucket reaches the other.

```text
ciproxy-xcode/
  xcode/<XCODE_XIP_NAME>
```

- `XCODE_XIP_NAME` and `XCODE_SHA256` come from `image/macos/pins.env`.
- A person uploads each archive once with an upload token.
- The build reads the archive with a read-only token, checks
  `XCODE_SHA256`, and keeps the archive out of the repository and the
  image. Decision 6 chooses how the build reads it.
- The bucket never gets a public domain.

### Settings and credentials

These names are fixed. Code, docs, and the operator use only these.

Cloudflare account `785233830ca9dfa2eb7c3561dd52f675`. The account ID is
not a secret. The buckets have the location hint WEUR, no jurisdiction,
no r2.dev access, and no custom domain. So the S3 endpoint is
`https://785233830ca9dfa2eb7c3561dd52f675.r2.cloudflarestorage.com`.

| R2 API token                             | Permission and scope                               | Store                                                                                                                                                                 |
| ---------------------------------------- | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ciproxy-main-preview-macos-images-read` | Object Read only, bucket `ciproxy-macos-images`    | Secret Manager secret `ciproxy-main-preview-r2-images` in project `ciproxy`                                                                                           |
| `ciproxy-production-macos-images-read`   | Object Read only, bucket `ciproxy-macos-images`    | Secret Manager secret `ciproxy-production-r2-images` in project `ciproxy`                                                                                             |
| `ciproxy-macos-images-publish`           | Object Read & Write, bucket `ciproxy-macos-images` | `~/ws-i/secrets/r2-publish.env` on the build Mac, mode 600. Source it before `publish.sh`, which reads `R2_PUBLISH_ACCESS_KEY_ID` and `R2_PUBLISH_SECRET_ACCESS_KEY`. |
| `ciproxy-xcode-read`                     | Object Read only, bucket `ciproxy-xcode`           | `~/ws-i/secrets/r2-xcode-read.env` on the build Mac, mode 600. Source it before `fetch.sh`, which reads `R2_XCODE_ACCESS_KEY_ID` and `R2_XCODE_SECRET_ACCESS_KEY`.    |

- Each Secret Manager version holds one JSON object:
  `{"access_key_id": "<token ID>", "secret_access_key": "<SHA-256 hex of
  the token value>"}`. The Cloudflare API gives the token ID and the
  token value. The secret access key is the SHA-256 of the value.
- Terraform creates each Secret Manager parent in the `data` root, as for
  the Boat snapshot. Add a version only after that data apply. A parent
  made by hand conflicts with the apply.
- Deploy copies the latest version into the Kubernetes Secret `r2-images`
  with the key `R2_IMAGES_KEY`. The pod reads it as the optional variable
  `R2_IMAGES_KEY`.
- GitHub environment variables in `main-preview` and `production`:
  - `MACOS_IMAGE_URL_PREFIX`:
    `https://785233830ca9dfa2eb7c3561dd52f675.r2.cloudflarestorage.com/ciproxy-macos-images`.
  - `MACOS_IMAGE_MANIFEST_PATH`: `releases/<image_id>/manifest.json`, or
    empty until the first publish.
- `publish.sh` and `fetch.sh` also read `R2_ACCOUNT_ID`.
- The scripts give the R2 keys to the AWS CLI only in its own process
  environment. They never print them.
- No GitHub secret is needed. Cal uploaded the Xcode archive with his own
  account token, so no Xcode upload token is needed.
- 23 November 2026: Cal made the publish and the Xcode read tokens. Their
  only store is an env file on the build Mac, in a folder with mode 700,
  because the build Mac has no password manager. Each file exports its two
  keys and `R2_ACCOUNT_ID`. Cal checked each scope: each token reads its
  own bucket and the other bucket refuses it.
- Cal makes the two environment read tokens after the data apply of H3
  creates their Secret Manager parents.

### The macOS release reader

- The control plane reads the macOS release only when
  `MACOS_IMAGE_URL_PREFIX`, `MACOS_IMAGE_MANIFEST_PATH`, and
  `R2_IMAGES_KEY` are all set. Otherwise it logs once that macOS images
  are off.
- It reads `manifest.json` and the sibling `downloads.json` with
  presigned URLs that expire after 5 minutes. It takes `image_id` and
  `root_disk_bytes` from the manifest, and the `macos` artifact from
  `downloads.json`.
- A failed read or a malformed key logs a warning, and the control plane
  starts without a macOS release.
- The presigned URL uses AWS Signature Version 4 with the region `auto`,
  the service `s3`, the signed header `host`, and `UNSIGNED-PAYLOAD`. A
  `FetchImage` URL expires after 2 hours.

### The resumable download

- The agent writes `images/<name>.tar.partial` and the sidecar
  `images/<name>.tar.partial.json`, which holds the image ID, the SHA-256,
  and the byte size. A partial file continues only when its sidecar
  matches the command.
- On resume, the agent hashes the bytes on disk, then sends
  `Range: bytes=<length>-`. A `206` answer must start at that length. A
  `200` answer starts the file again from zero.
- Inside one command, the agent retries a broken stream with `Range` a
  few times, with a short wait. An HTTP error such as `403` for an expired
  URL ends the command and keeps the partial file. The next `FetchImage`
  brings a fresh URL.
- Recovery at start keeps a partial file with a valid sidecar. It still
  removes a finished archive that waits for extraction, as today.
- One download runs at a time. A second `FetchImage` for the same image
  waits for the first.
- Checked on 23 November 2026 on the test Mac: R2 finishes a running
  download after its presigned URL expires. A GET URL with a 30 s expiry
  streamed all 2,965,708,631 bytes of the Xcode archive in 188 s with HTTP
  200. A new request with the same URL 35 s later got 403. So a 2-hour URL
  and the `Range` resume of decision 2 D are enough.
- After the size and the SHA-256 match, the partial file becomes
  `images/<name>.tar`, and publication continues as today.

## Pull requests

- PR H2, this workspace: Milestone 2, merged as #129.
- PR H3, sub-workspace "H3 macOS release from R2": Milestones 1 and 3, as
  #130, which targets `main`.
- PR H5, sub-workspace "H5 resumable image download": Milestone 5, from
  `main`.
- PR H4, sub-workspace "H4 publish and Xcode fetch": Milestones 4 and 6,
  stacked on PR #113, because `image/macos/` comes with that pull request.
  It merges after PR H3, because only H3 makes the deploy read
  `MACOS_IMAGE_MANIFEST_PATH`.

Merge order: H2, then H3. PR #113, then H4 after H3. H5 has no order.

## Decisions

The decisions are numbered in the order of the message to the parent
workspace. Cal answered all six on 23 November 2026. Each item gives its
outcome.

### 1. How a Mac proves it may download

- **Severity:** High. **Effort:** Medium.
- **Source:** Design.
- **Category:** Security.
- **Location:** `crates/ciproxy-control-plane/src/image_release.rs`
- **Terms:**
  - **Presigned URL:** an S3 URL with an AWS Signature Version 4 in its query.
    It allows one operation on one object until it expires.
- **Background:** The macOS image holds Xcode. The user has approval to give it
  to customers. The Linux image sits in a private GCS bucket, and the control
  plane signs a Cloud CDN URL for each send. The agent checks the size and the
  SHA-256 that the control plane sends.
- **Problem:** R2 has no Cloud CDN signed URLs. We must choose how an R2
  download is limited to the customers' Macs.
- **Impact if no action:** No Mac can download an image.
- **Root question:** How does only a ciproxy Mac get the macOS image, at low
  cost and with little to run?
- **Options:**
  - **A.** Public bucket on a custom domain, such as `images.ciproxy.dev`. The
    SHA-256 gives integrity. Anyone with the path can download Xcode, which can
    go past the approval. Effort: Small.
  - **B.** Private bucket. The control plane signs a presigned GET URL for each
    `FetchImage`, with a read-only R2 token scoped to the image bucket. The URL
    goes to `<account>.r2.cloudflarestorage.com`. No Worker and no paid plan.
    Effort: Medium.
  - **C.** Private bucket behind a custom domain with a Cloudflare Worker that
    checks an HMAC token of the same form as the Cloud CDN token. The control
    plane reuses its CDN signer. A Worker to deploy, to pass `Range` through,
    and to maintain. Effort: Large.
  - **D.** Custom domain with a Cloudflare WAF rule that checks a timed HMAC
    token (`is_timed_hmac_valid_v0`). No code, but it needs a Pro plan or higher
    on the `ciproxy.dev` zone, and the rule lives outside the repository.
    Effort: Medium.
  - **E.** Option B, but a small maintained crate, such as `rusty-s3`, makes the
    presigned URL. Less signing code to own, one new dependency. Effort: Small.
- **Left out:** R2 credentials on each Mac: a customer machine would hold a key
  for every image. Short-lived R2 temporary credentials for each Mac: more parts
  than B for one GET of one object. Cloudflare Access service tokens: each Mac
  would hold a long-lived shared secret. A proxy through the control plane: GCP
  egress costs about USD 5 to 7 for each 60 GB download, and the bytes load the
  control plane.
- **Wider fix:** None. Each option sets the access path for all macOS releases.
- **Previous examples:** The Cloud CDN signer in
  `crates/ciproxy-control-plane/src/image_signing.rs`.
- **Recommendation:** B. It keeps Xcode private, needs only one bucket and one
  read token, and follows the Linux pattern of a URL per send. With B and E, the
  download host is `<account>.r2.cloudflarestorage.com`, so a customer firewall
  must allow that host. E is a sub-choice of B.
- **Outcome:** 23 November 2026, Cal chose B. A private bucket and a presigned
  URL for each send.
- **Action:** ask the user, because the options trade cost, privacy of Xcode,
  and parts to run, and no option is clearly best.

### 2. Resume of a 30 to 60 GB download

- **Severity:** High. **Effort:** Medium.
- **Source:** Design.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/platform/macos/images.rs`, `download`
- **Background:** The agent on a Mac downloads `macos.tar` in one HTTP stream,
  then checks the size and the SHA-256. A failure deletes nothing but starts
  again from zero at the next `FetchImage`. The control plane sends a fresh URL
  after each reported result. At 100 Mbit/s, 60 GB takes about 80 minutes. A
  Linux URL expires after 2 hours.
- **Problem:** One broken connection or one restart of the agent loses all bytes
  so far. A slow line may never finish.
- **Impact if no action:** A Mac on a slow or unstable line never gets an image,
  and runs no macOS jobs.
- **Root question:** How does a Mac finish a very large download over an
  unreliable line?
- **Options:**
  - **A.** Change nothing. One stream, and a fresh start after each failure.
    Effort: Small.
  - **B.** The agent keeps the partial file for the image ID and SHA-256, and
    continues it with an HTTP `Range` request. It retries inside one command,
    and continues across commands and agent restarts. On resume it hashes the
    bytes on disk again, about 20 s for 60 GB. The control plane signs macOS
    URLs for 12 hours. Effort: Medium.
  - **C.** Split the image into parts of about 1 GB, each with its own SHA-256,
    and download the parts in turn. This changes the bundle form of workstream I
    and the publish step. Effort: Large.
  - **D.** Option B, but the URL keeps the 2-hour expiry of Linux URLs. The
    agent retries with `Range` on the same URL while it is valid. Then the
    command fails, and the next `FetchImage` brings a fresh URL, as today. The
    partial file carries over. A first test checks that R2 does not stop a
    running stream at the expiry. Effort: Medium.
  - **E.** A Swift helper gives the download to a background `NSURLSession`,
    which keeps resume data across restarts. A new bridge from Swift to Rust and
    a new test harness. Effort: Large.
- **Left out:** A longer URL alone: it does not save the bytes after a break.
  Parallel `Range` parts: a speed change that can come later on top of B or D.
- **Wider fix:** D or B. Each serves every later large image too.
- **Previous examples:** `plans/follow-ups/repeat-install-image-download.md`,
  the same restart from zero for the Linux image.
- **Related:** Item 1: B and D of item 1 pass `Range` through. C of item 1 must
  pass it through too.
- **Recommendation:** D. The options check changed this from B. D keeps the
  resume of B and uses the existing re-send for fresh URLs, so a leaked URL to
  Xcode stays valid for 2 hours, not 12. B and D both change agent recovery,
  which deletes each `*.tar` at start today. They also stop a second
  `FetchImage` for the same image from writing the same partial file.
- **Outcome:** 23 November 2026, Cal chose D. Resume with `Range`, with the
  2-hour URL.
- **Action:** ask the user, because the resume behaviour is new agent behaviour
  that no requirement covers, and it touches the area of workstream A.

### 3. Which macOS release a Mac gets

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Docs or spec.
- **Location:** `crates/ciproxy-control-plane/src/application.rs`
- **Background:** The Linux release is pinned by the GitHub environment variable
  `JOB_IMAGE_MANIFEST_PATH`. The control plane reads it at start. A new Linux
  release needs a deploy. The Linux pin also keeps the Boat snapshot and the
  machine image on one release. The macOS image has no Boat snapshot.
- **Problem:** We must choose how the control plane learns the newest macOS
  release.
- **Impact if no action:** No macOS release reaches a Mac.
- **Root question:** How does a new macOS release reach the Macs of each
  environment, with a way back?
- **Options:**
  - **A.** A pinned GitHub environment variable `MACOS_IMAGE_MANIFEST_PATH`,
    read at start, as for Linux. A new release needs a variable change and a
    deploy. Rollback is the old value. Effort: Small.
  - **B.** Publish writes a pointer object `current.json` for each environment.
    The control plane reads it again every 10 minutes. A release needs no
    deploy. A bad publish reaches the Macs within minutes. Effort: Medium.
  - **C.** A hosted admin setting in the store, changed by an operator command.
    It needs a new table and an admin path. Effort: Large.
  - **D.** Pin the path for each environment in a file in the repository, such
    as the deploy values. A pull request changes it, and a revert rolls back. A
    release still needs a deploy, and the pin then differs from the Linux GitHub
    variable. Effort: Small.
- **Left out:** The newest folder by listing the bucket: a half or a test upload
  could win.
- **Wider fix:** None. A is already the Linux pattern.
- **Previous examples:** `JOB_IMAGE_MANIFEST_PATH` in
  `deploy/scripts/deploy.py`.
- **Recommendation:** A. It reuses the deploy path and the rollback of Linux,
  and a macOS release is rare. D is the choice for a reviewed record of each
  release in the repository.
- **Outcome:** 23 November 2026, Cal chose A. The GitHub variable
  `MACOS_IMAGE_MANIFEST_PATH`.
- **Action:** ask the user, because B saves a deploy for each release, and the
  user chooses that trade.

### 4. How the R2 read key reaches the control plane

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Security.
- **Location:** `deploy/helm/ciproxy/templates/control-plane.yaml`,
  `deploy/terraform/modules/foundation/variables.tf`
- **Background:** The pod mounts its secrets from Secret Manager through the CSI
  driver. A listed secret without a version stops the mount, and so the pod. The
  optional Boat snapshot uses another path: deploy copies the latest Secret
  Manager version into a Kubernetes Secret, and the pod reads it as an optional
  variable.
- **Problem:** The R2 read token needs a path into the pod. The macOS image must
  stay optional while the key does not exist.
- **Impact if no action:** The control plane cannot sign R2 URLs.
- **Root question:** How does the hosted process get an optional secret without
  a risk to its start?
- **Options:**
  - **A.** Add `r2-images-key` to the mounted CSI secrets in Terraform. Every
    pod fails to start until an operator adds a version in each environment.
    Effort: Small.
  - **B.** The Boat path. Terraform adds the parent `ciproxy-<env>-r2-images`.
    Deploy copies its latest version into an optional Kubernetes Secret. Without
    a version, macOS stays off and Linux runs. Effort: Small.
  - **C.** A GitHub environment secret that deploy writes into a Kubernetes
    Secret. Simple for the operator, but the key passes through GitHub Actions,
    unlike the other runtime secrets. Effort: Small.
  - **D.** Mount the key through the CSI driver only when a GitHub variable,
    such as `R2_IMAGES_KEY_VERSION`, names a version, as `CDN_SIGNING_KEY_NAME`
    does for the CDN key. The key stays out of Kubernetes objects. A wrong value
    stops the pod, and so Linux too. Effort: Small.
  - **E.** The control plane reads the latest version from Secret Manager
    through the GCP API at start. `NOT_FOUND` turns off macOS only. A new Google
    client dependency and a second way to load secrets. Effort: Medium.
- **Left out:** None.
- **Wider fix:** None. B reuses an existing helper.
- **Previous examples:** `deploy/scripts/boat_secret.py`.
- **Recommendation:** B. It is the only option where a missing or wrong key
  never stops the pod. B copies the key into a Kubernetes Secret, as the Boat
  snapshot does. D keeps it out of Kubernetes objects.
- **Outcome:** 23 November 2026, Cal chose B. The Boat path, with
  `ciproxy-<env>-r2-images`.
- **Action:** ask the user, because it adds a Secret Manager secret and an
  operator step in each environment.

### 5. Who runs the publish step

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Process.
- **Location:** `image/macos/` (new `publish.sh`)
- **Background:** The macOS build runs by hand on an Apple Silicon Mac, as
  `image/macos/README.md` in PR #113 says. GitHub-hosted macOS runners cannot
  run the nested VM that the build needs. The Linux image is published by
  `image/publish.sh` from the build machine.
- **Problem:** We must choose where the publish step runs and where its write
  token lives.
- **Impact if no action:** No image reaches R2.
- **Root question:** How does a finished build reach the bucket safely?
- **Options:**
  - **A.** A script, `image/macos/publish.sh`, that a person runs on the build
    Mac. The write token sits in environment variables from a password manager,
    not in GitHub. Effort: Small.
  - **B.** A `workflow_dispatch` workflow on a self-hosted runner on the build
    Mac, with the token as an environment secret. This repository is public, so
    a self-hosted runner needs strict limits against fork pull requests. Effort:
    Medium.
  - **C.** A workflow on ciproxy's own macOS label, which builds and publishes
    on a Mac that ciproxy runs. It needs the macOS work to be complete first. A
    ciproxy macOS job runs in a VM, and the build starts its own macOS VM, which
    Virtualization.framework probably does not allow. Effort: Large.
  - **D.** Option A, but for each publish the operator mints R2 temporary
    credentials from the publish token. They allow only writes under
    `releases/<image_id>/` and expire after a few hours. One more step for each
    publish. Effort: Small.
- **Left out:** Upload from a GitHub-hosted runner: the artifacts are on the
  build Mac, and 60 GB is too large to move twice.
- **Wider fix:** None.
- **Previous examples:** `image/publish.sh`.
- **Recommendation:** A. The build is already manual, and a script adds no
  runner and no secret to the public repository. D is an optional improvement on
  A. C is probably blocked by nested virtualization.
- **Outcome:** 23 November 2026, Cal chose A. `image/macos/publish.sh`, run by a
  person on the build Mac.
- **Action:** ask the user, because B puts a write secret in GitHub, and the
  user chooses where secrets live.

### 6. How the build reads the private Xcode copy

- **Severity:** Medium. **Effort:** Small.
- **Source:** Design.
- **Category:** Process.
- **Location:** `image/macos/fetch.sh` (PR #113)
- **Background:** Apple serves Xcode only after an Apple Account sign-in. Today
  a person downloads the `.xip` and names its folder in `XCODE_DIR`. `pins.env`
  pins `XCODE_XIP_NAME` and `XCODE_SHA256`. GitHub's template reads
  `XCODE_INSTALL_STORAGE_URL` with a signed Azure token for a whole container.
- **Problem:** The private bucket of the user decision needs an upload path and
  a read path for the build.
- **Impact if no action:** A new or rebuilt build Mac needs a manual Xcode
  download with an Apple Account.
- **Root question:** How does a build get the pinned Xcode archive with no Apple
  sign-in?
- **Options:**
  - **A.** `fetch.sh` downloads `xcode/<XCODE_XIP_NAME>` with the AWS CLI and a
    read-only token scoped to `ciproxy-xcode`, then checks `XCODE_SHA256`.
    `XCODE_DIR` stays as a local override. Effort: Small.
  - **B.** An operator makes a presigned URL for each build, valid for up to 7
    days, and passes it as `XCODE_INSTALL_STORAGE_URL`. No long-lived token on
    the build Mac, but each build needs a manual step. `fetch.sh` does not read
    that variable today, and a presigned URL names one archive, not a container.
    Effort: Small.
  - **C.** Change nothing. A person downloads Xcode on each build Mac. Effort:
    Small.
  - **D.** Leave `fetch.sh` as it is. The README gives one `aws s3 cp` command
    that copies the archive from `ciproxy-xcode` into `XCODE_DIR` with the
    read-only token. One manual command for each build Mac and each Xcode
    version. Effort: Small.
- **Left out:** None.
- **Wider fix:** None.
- **Previous examples:** `XCODE_INSTALL_STORAGE_URL` in the upstream Packer
  template.
- **Related:** Item 5: the same build Mac holds the publish token.
- **Recommendation:** A. A read-only token for one bucket is a small risk, and
  it lets a build run with no person. D is the choice for no change to the
  script of workstream I.
- **Outcome:** 23 November 2026, Cal chose A. `fetch.sh` reads the archive from
  `ciproxy-xcode`.
- **Action:** ask the user, because it puts a long-lived token on the build Mac,
  and the build script belongs to workstream I.

## Milestone 1: Documentation and protocol

Depends on: decisions 1, 2, and 3.

- [x] State the hosting of the macOS image in
      `docs/protocol/agent-images.md`: a private store, a URL per send, an
      image for Macs only, and the resume behaviour. (resume behaviour: PR
      #127)
- [x] Add the macOS settings to `deploy/runtime.md` and `deploy/setup.md`.

## Milestone 2: One image for each platform

Depends on: none.

- [x] Hold at most one release for each platform in `Config` and
      `MachineRuntime`, and offer each machine the release of its platform.
- [x] Test that a Mac gets the macOS image and a Linux machine the Linux
      image, and that a machine without a release of its platform gets
      nothing.

## Milestone 3: Read the macOS release from R2

Depends on: 1, 2, and decisions 1, 3, and 4.

- [x] Add the presigned URL signer with the AWS test vector.
- [x] Add `R2Releases` and the optional macOS settings.
- [x] Wire the settings and the optional credentials through
      `deploy/scripts/deploy.py`, the chart, and the deploy workflows.

## Milestone 4: Publish

Depends on: PR #113 and decision 5.

- [ ] Add `image/macos/publish.sh` and its section in
      `image/macos/README.md`.

## Milestone 5: Resume in the agent

Depends on: decision 2.

- [ ] Resume the macOS download as decision 2 says, with tests.

## Milestone 6: Xcode archive bucket

Depends on: PR #113 and decision 6.

- [ ] Read the Xcode archive from `ciproxy-xcode` in the build, as
      decision 6 says.

## Milestone 7: Verification, commit, review, and pull request

Depends on: every other milestone.

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the diff with the `implementation-review` skill and apply the
      fix rule.
- [ ] Delete this plan when the `plans` skill says.
- [ ] Open the pull requests.

## Post-merge follow-up (non-blocking)

- [ ] Publish the first release image and pin it in main preview. The
      parent runs the Mac steps.
