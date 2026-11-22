# Privacy retention periods have no protocol home

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, the
  blocked wider fix of review findings 6 and 10.
- **Location:** `docs/protocol/security.md`, "Data retention".
  `ts/site/src/data/legal-privacy.ts`, "Data retention".
  `deploy/terraform/modules/foundation/data.tf`, `backup_configuration`.
  `crates/ciproxy-store-pg/src/billing/quotes.rs`, the quote pruning.
- **Reason:** The fix adds requirements to a protocol doc, and only the user
  can add a requirement.

- **Background:**
  - The privacy policy states periods for the database backups, the
    operational logs, the billing records, the quotes, and the sign-in
    records.
  - "Data retention" in `docs/protocol/security.md` decides only the job
    metadata, the savings totals, the machine reports, and the person
    records. `ts/site/src/data/legal.ts` says that its limits follow that
    doc.
  - The other periods come from the code and the deployment: 14 daily Cloud
    SQL backups in the multi-region `us`, Cloud Logging's default of 30
    days, billing records that go with the workspace, quote pruning when a
    new quote is saved, and sign-in records that go at sign-out or soon
    after the sign-in ends.
  - The review of the legal pages found two such periods wrong in the first
    text: the backups and the quotes.
- **Problem:** A change of the backup count, the backup location, the log
  retention, or the quote pruning can make the privacy policy wrong, and no
  review checks the change against it.
- **Impact if no action:** The published policy can promise a period that
  the service no longer keeps.
- **Root question:** Where do the retention promises of the privacy policy
  live, so that a change of the code or the deployment is checked against
  them?
- **Options:**
  - **A.** Keep the periods only in the privacy text. Effort: Small.
  - **B.** Put each period in "Data retention" of
    `docs/protocol/security.md`, with a decision line. Link to it from a
    comment above `backup_configuration`. The doc has 317 lines, so split it
    into linked pages in the same change. Effort: Medium.
  - **C.** B, and set `location = "us"` in the Cloud SQL backup configuration,
    so that the code states the backup location. Effort: Medium.
  - **D.** Add reminders that point to the privacy policy: a line in
    `crates/ciproxy-store-pg/src/billing/README.md` and a comment in
    `data.tf`. They are not reviewed rules. Effort: Small.
- **Left out:** A value that Terraform and the site share stays out, because
  a change of it would change the published policy without the owner's
  approval and a new effective date.
- **Wider fix:** B. The post-push review then checks the code and the
  deployment against one requirement.
- **Previous examples:** Review findings 6 and 10 of the branch
  `docs/legal-pages-firna`.
- **Recommendation:** B. It gives the periods one reviewed home, as the other
  retention limits already have.
- **Action:** ask the user, because B adds requirements to a protocol doc.
