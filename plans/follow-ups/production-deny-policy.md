# Merged changes can delete production data

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-05, from the branch
  `calummoore/ci-proxy-load-balancer`, the answer D to item 56 of
  `plans/ciproxy-mvp.md`.
- **Location:** `deploy/terraform/modules/foundation/`, and the IAM policy
  of the GCP project `ciproxy`.
- **Reason:** The user chose option D, plan, check, and promote, on
  5 November 2026. The recommendation also proposed this guard before
  launch, and the user did not choose it.

- **Terms:**
  - **Deny policy:** a GCP rule that refuses chosen permissions to chosen
    principals, whatever roles they hold. Its admin role lives on a GCP
    organization.
  - **Apply identity:** the service account that applies the saved
    Terraform plans from CI. It can change the access rules of the
    project.
- **Background:**
  - CI applies every Terraform root after each merge, as the answer D to
    item 56 says.
  - A policy in the pipeline refuses each destructive change that has no
    approval file in the merge.
  - The database and the secrets get deletion protection in GCP.
  - The production project holds the customer data: the Cloud SQL
    database, the KMS key that encrypts each Boat key, and the secrets.
  - When Terraform destroys the Boat KMS key, the data that it encrypted
    becomes unreadable after a 30-day window.
  - The apply identity can change the access rules of the project, so it
    can also remove the deletion protection.
- **Problem:** A merged change with an approval file, or a broken policy
  check, can still delete the customer data through the apply identity.
- **Impact if no action:** One wrong merge into `main` can delete the
  production database or the Boat KMS key.
- **Root question:** How does production keep its customer data when a
  merged change tries to delete it?
- **Options:**
  - **A.** Keep the pipeline policy and the deletion protection only.
    Effort: Small.
  - **B.** Add a GCP deny policy on the project `ciproxy` that refuses the
    irreversible deletes to every principal outside an emergency group: the
    database, the KMS key versions, the secrets, the buckets, the DNS zone,
    the cluster, and the federation pool. An operator sets it once. No
    merged change can remove it. It needs a GCP organization above the
    project. Effort: Medium.
  - **C.** Keep the backups of the database and the KMS key in a second
    project that CI cannot reach. Effort: Medium.
- **Left out:** Required reviewers on each production run, because a
  person must act after each merge, which the user rejected.
- **Wider fix:** B. GCP itself refuses the deletes, whatever the pipeline
  does.
- **Previous examples:** None found.
- **Recommendation:** B, before launch, when the projects sit in a GCP
  organization. Check that first with `gcloud projects describe ciproxy`.
- **Action:** ask the user, because B adds a new refusal at the GCP level,
  and it needs the rights of the GCP organization admin.
