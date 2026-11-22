# Provider roles in the privacy policy lack sources

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, the
  split-off wider fix of review finding 8 and a problem that the check of
  review finding 4 found.
- **Location:** `ts/site/src/data/legal-privacy.ts`, "Who processes data for
  ciproxy" and "Your rights". `ts/site/README.md`, "Rules for the legal
  pages".
- **Reason:** The fix needs a review of the current agreements of six
  providers, which is separate work from the legal text change.

- **Background:**
  - The privacy policy names Google Cloud, Cloudflare, and Google Workspace
    as processors, and Stripe as a processor for its billing services and a
    separate controller for its own uses. It names GitHub and Boat as
    separate controllers.
  - It names the transfer safeguards: the UK Extension to the EU-US Data
    Privacy Framework, the standard contractual clauses with the UK
    Addendum, and, where the EU GDPR applies, the EU-US Data Privacy
    Framework.
  - The review of the legal pages found the first Stripe role wrong. The
    text had named Stripe only as a separate controller.
  - No repository doc records where Boat processes data or which contract
    covers it. Boat runs in the customer's own Boat account.
- **Problem:** The roles and the safeguards come from common practice, not
  from each provider's current agreement, so a role or a safeguard can be
  wrong.
- **Impact if no action:** A person can send a request to the wrong party, or
  rely on a safeguard that does not apply.
- **Root question:** Where do the provider roles and the transfer safeguards
  of the privacy policy come from, and how does a later change keep them
  right?
- **Options:**
  - **A.** Keep the current text. Effort: Small.
  - **B.** Read the current data processing agreement or privacy policy of
    each provider. Correct the text where it differs. Name each document,
    its section, and its date in the pull request. Add a rule to "Rules for
    the legal pages" in `ts/site/README.md` that requires the same for each
    later change. Effort: Medium.
  - **C.** B, and show a table of providers on the privacy page, with the
    role, the request contact, and the safeguard of each. It needs a mockup
    change first. Effort: Large.
- **Left out:** A separate provider page stays out. It only moves the table
  of C to a second page.
- **Wider fix:** B. The README rule makes each later change name its source.
- **Previous examples:** Review findings 4 and 8 of the branch
  `docs/legal-pages-firna`.
- **Recommendation:** B. It checks each claim against its source at Medium
  effort.
- **Action:** fix after merge, apply B, because it needs a source review of
  six providers. Each corrected sentence is a new version of the policy and
  needs the owner's approval.
