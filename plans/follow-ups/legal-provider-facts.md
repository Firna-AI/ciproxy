# Some provider facts in the privacy policy lack sources

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, the
  split-off wider fix of review finding 8, a problem that the check of
  review finding 4 found, and the source rule of review finding 13.
- **Location:** `ts/site/src/data/legal-privacy.ts`, "Who processes data for
  ciproxy".
- **Reason:** The fix needs a review of the current agreements of three
  providers and the transfer safeguards, which is separate work from the
  legal text change.

- **Background:**
  - The privacy policy names Google Cloud, Cloudflare, and Google Workspace
    as processors, and Stripe as a processor for its billing services and a
    separate controller for its own uses. It names GitHub and Boat as
    separate controllers for their own accounts and uses, which act for the
    customer for the content of its organizations and sandboxes.
  - It names the transfer safeguards: the UK Extension to the EU-US Data
    Privacy Framework, the standard contractual clauses with the UK
    Addendum, and, where the EU GDPR applies, the EU-US Data Privacy
    Framework.
  - The review of the legal pages found the first Stripe, GitHub, and Boat
    roles wrong. The text now takes them from their sources, which comments
    beside the sentences name: Stripe's Data Processing Agreement of 28
    September 2026, section 2; GitHub's Data Protection Agreement of October
    2025, sections 3.A and 3.G; and Boat's privacy policy, version
    2026-10-12, PP-2 and PP-10.
  - `ts/site/README.md` now requires a source comment for each provider role
    and each transfer safeguard.
  - Google Cloud, Cloudflare, and Google Workspace have no source comment
    yet, and the transfer safeguards have none.
- **Problem:** The roles of Google Cloud, Cloudflare, and Google Workspace,
  and the transfer safeguards, come from common practice, not from each
  provider's current agreement, so one of them can be wrong.
- **Impact if no action:** A person can send a request to the wrong party, or
  rely on a safeguard that does not apply.
- **Root question:** Which current source supports each remaining provider
  role and each transfer safeguard of the privacy policy?
- **Options:**
  - **A.** Keep the current text. Effort: Small.
  - **B.** Read the current data processing agreements of Google Cloud,
    Cloudflare, and Google Workspace, and the sources of the transfer
    safeguards. Correct the text where it differs, and add a source comment
    beside each sentence, as the README rule says. Effort: Small.
  - **C.** B, and show a table of providers on the privacy page, with the
    role, the request contact, and the safeguard of each. It needs a mockup
    change first. Effort: Large.
- **Left out:** A separate provider page stays out. It only moves the table
  of C to a second page.
- **Wider fix:** None. The README rule is already in force.
- **Previous examples:** Review findings 4 and 8 of the branch
  `docs/legal-pages-firna`.
- **Recommendation:** B. It checks each remaining claim against its source.
- **Action:** fix after merge, apply B, because it needs a source review of
  three providers. Each corrected sentence is a new version of the policy and
  needs the owner's approval.
