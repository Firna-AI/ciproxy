# Production does not exist yet

- **Severity:** High. **Effort:** Large. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the deployment and launch milestones of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** `deploy/launch.md`, `deploy/setup.md`,
  `deploy/first-deploy.md`, `deploy/runtime.md`, `deploy/dns.md`,
  `deploy/image-workflow-bootstrap.md`, and `deploy/runtime-identity.md`.
- **Reason:** Production deploys only from protected `main`, so its setup
  starts after the launch pull request merges into `main`.

- **Terms:**
  - **Main preview:** the `main-preview` environment at
    `br-main.preview.*.ciproxy.dev`. It deploys each push of the primary
    branch.
  - **Launch guide:** `deploy/launch.md`, the one-time operator procedure
    that hands the shared GCP trust to production.
  - **Job image release:** the published VM image of the runners, with its
    manifest under the image bucket of an environment.
  - **Boat:** the overflow provider. ciproxy rents Boat sandboxes with the
    key of the customer when the machines of the customer are busy.
- **Background:**
  - Production runs in the GCP project `ciproxy`, in the namespace
    `ciproxy-production`. Regional resources are in `us-central1`. The
    cluster, the network, and the image registry are shared with the main
    preview.
  - Production deploys on each merge into `main`, after the main preview
    run of the same commit passes. The user chose this on 4 November 2026.
  - On 17 November 2026 the repository variable
    `PRODUCTION_DEPLOY_ENABLED` was `false`. GitHub had no `production`
    environment.
  - On the same date the main preview had `CONTROL_PLANE_RUNTIME_READY`
    and `PUBLIC_UPTIME_ENABLED` set to `true`, and the CDN key
    `ciproxy-main-preview-images-v1`. It ran agent `ciproxy-agent-v0.1.2`,
    a pre-release, and the job image release `2026-11-17-4a950ae`.
  - The user chose on 8 November 2026 that the image-publishing
    credentials bind to the reusable image workflow of the primary branch
    until the launch, and move to protected `main` at the launch.
- **Task:**
  1. Wait for the production GitHub App and the Stripe live mode of
     `production-accounts.md`.
  2. Follow `deploy/launch.md` in its order. Its "Prepare" list includes
     the move of the image-publishing credentials to protected `main`, as
     `deploy/image-workflow-bootstrap.md` says under "Move the binding to
     main at launch".
  3. Complete the first production data run, the release token, the DNS
     delegation of `app` and `get` from Cloudflare, the edge, the apex
     record, the CDN key, the HTTPS checks, and the uptime alerts. The
     launch guide links each step.
  4. Turn on the runtime as `deploy/runtime.md` says. Set
     `CIPROXY_APP_ID`, `CIPROXY_APP_CLIENT_ID`, and `CIPROXY_APP_SLUG` of
     the production App. Add a version to each secret
     `ciproxy-production-<file>` from a private file: the App key, the
     webhook secret, the OAuth secret, the Stripe key, the Stripe webhook
     secret, and the session key.
  5. Publish a job image release for production with its Boat shared
     snapshot. Set `JOB_IMAGE_MANIFEST_PATH` and the Boat snapshot secret
     of production. Then set `CONTROL_PLANE_RUNTIME_READY` to `true`.
  6. Publish the first full agent release, which is not a pre-release.
     Production follows only full releases. `release-server-prerelease-channel.md`
     gives the check of the release channel.
- **Impact if no action:** ciproxy has no production environment, so no
  customer can use it.
- **Needs:** The launch pull request merged into `main`. Operator access:
  owner of the GCP project `ciproxy`, the Cloudflare DNS of `ciproxy.dev`,
  and admin of `Firna-AI/ciproxy`. The production App and Stripe live keys.
- **Done when:** A production run from `main` passes every job. The
  verify step of the launch probe passed, so the prelaunch accounts stay
  disabled. `https://app.ciproxy.dev`, `https://get.ciproxy.dev`, and
  `https://ciproxy.dev` answer over HTTPS. Uptime alerts are on. The
  control plane is ready with the production App and the Stripe live keys.
- **Related:** `production-deny-policy.md` is an open decision on a guard
  for production data. Decide it before the first production apply.
  `production-smoke-test.md` follows this task.
  `shared-saved-runner-group.md` and
  `environments-share-organization-jobs.md` hold choices for the first
  time that production serves an organization that the main preview
  already serves, such as `Firna-AI`.
