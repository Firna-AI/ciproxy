# Labelled pull requests have no preview

- **Severity:** Low. **Effort:** Large. **Confidence:** Medium. **Harder
  later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-04, from the branch
  `calummoore/ci-proxy-load-balancer`, the user's answer on the preview
  host names.
- **Location:** `deploy/`, and `.github/workflows/`.
- **Reason:** On 4 November 2026 the user chose the main preview now, and
  previews of pull requests after the control plane can run.

- **Background:**
  - ciproxy deploys a main preview at `br-main.preview.ciproxy.dev`, with
    the dashboard at `br-main.preview.app.ciproxy.dev` and the downloads at
    `br-main.preview.get.ciproxy.dev`.
  - `mokly-cloud` and `futex-ai/accounting` also give each same-repository
    pull request with the `preview` label its own environment at
    `pr-<number>.preview.<domain>`. It runs in its own namespace of the
    shared cluster, with a small Postgres, and the workflow deletes it when
    the label goes or the pull request closes.
  - In `mokly-cloud`, the workflow builds the images without cloud
    credentials, and wildcard DNS records point `*.preview.<domain>` to one
    shared ingress address.
  - A preview of ciproxy's control plane also needs a GitHub App
    registration that does not clash with the main preview.
- **Task:** Add previews of labelled same-repository pull requests at
  `pr-<number>.preview.ciproxy.dev`, `pr-<number>.preview.app.ciproxy.dev`,
  and `pr-<number>.preview.get.ciproxy.dev`, as `mokly-cloud` does. Each gets
  its own namespace and database, and the workflow deletes it at unlabel or
  close.
- **Impact if no action:** Reviewers see a pull request in a running
  environment only after its merge.
- **Needs:** The main preview set up, and a control plane that runs, after
  Milestone 14 of `plans/ciproxy-mvp.md`.
- **Done when:** A labelled pull request gets its three hosts with a
  working sign-in, and removing the label deletes its namespace, its data,
  and its DNS records.
