# Changed terms never reach unattended workspaces

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, the
  blocked wider fix of review finding 7.
- **Location:** `ts/site/src/data/legal-terms.ts`, the sections "Changes to
  these terms" and "Plans and payment".
- **Reason:** The fix changes the acceptance rule that the user took from
  the conductive terms, so only the user can choose it.

- **Background:**
  - The terms say that a changed version applies from the next sign-in, and
    that a sign-in accepts it. The user chose this rule from the conductive
    terms on 22 November 2026.
  - A sign-in lasts at most 30 days. Jobs keep running without any sign-in,
    because the agent and the GitHub listener need no person.
  - So the version that a workspace accepted last keeps applying while
    nobody signs in. The terms now say so.
  - On conductive, each use needs a sign-in of at most 7 days, so a change
    applies within 7 days there.
  - ciproxy sends no email, as `docs/protocol/security.md` records. Stripe
    emails the receipts of paid workspaces.
  - Price changes follow their own rule: an email to the billing address in
    Stripe at least 30 days before the new price applies.
- **Problem:** A workspace that nobody signs in to stays on an old version
  of the terms with no end date.
- **Impact if no action:** Firna Ltd cannot apply a changed version to such a
  workspace, also when the change matters for the service.
- **Root question:** Which event makes a new version of the terms bind a
  customer whose jobs run without any sign-in?
- **Options:**
  - **A.** Keep the next-sign-in rule. Effort: Small.
  - **B.** A new version applies 30 days after we publish it, or at an
    earlier sign-in. Continued use after that date accepts it. UK consumer
    law can treat a one-sided change without direct notice as unfair.
    Effort: Small.
  - **C.** Paid workspaces get an email to their billing address in Stripe,
    and they follow the rule of B. Free workspaces keep the next-sign-in
    rule. An operator sends the email, because ciproxy has no email sender.
    Effort: Medium.
  - **D.** After a set time, new runners of a workspace wait until a person
    with authority accepts the new version in the dashboard. This stops the
    CI of unattended workspaces, against the soft-limit decision that a
    pricing limit never stops a customer's CI. Effort: Large.
- **Left out:** A shorter sign-in fails the goal, because an unattended
  workspace has no sign-in to shorten.
- **Wider fix:** B. It ties every change to its publication date, so it still
  holds when the sign-in paths change.
- **Previous examples:** None found.
- **Related:**
  - `terms-notice-on-every-sign-in.md`: the same acceptance rule. Its option
    C records the accepted version, which gives proof of the version that
    binds a workspace.
  - `terms-acceptance-parties.md`: whose sign-in binds the customer. Its
    option E uses option B here.
- **Recommendation:** C. Paid workspaces, which have the most at stake, get a
  direct notice, and the price change rule already uses the same email.
- **Action:** ask the user, because every option except A changes the
  acceptance rule that the user took from conductive.
