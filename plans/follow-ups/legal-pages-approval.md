# The terms and privacy policy are drafts

- **Severity:** High. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the launch milestone of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** `ts/site/src/pages/terms.astro`,
  `ts/site/src/pages/privacy.astro`,
  `ts/site/src/components/legal/LegalPage.astro`,
  `ts/site/src/components/legal/legal.css`, and `ts/site/README.md`.
- **Reason:** Only the user can complete the legal details and approve the
  texts.

- **Terms:**
  - **Placeholder:** a detail in square brackets, such as
    `[effective date]`, that the owner must complete.
- **Background:**
  - The site pages `/terms/` and `/privacy/` hold the complete draft text
    of the terms of service and the privacy policy.
  - A shared draft note at the top of each page says that the text is a
    draft.
  - The privacy policy states the data retention limits of "Data retention"
    in `docs/protocol/security-data.md`. The limits come from
    `ts/site/src/data/legal.ts`.
  - The site rules forbid a guess or the current date in place of a
    placeholder.
- **Task:**
  1. The user completes each placeholder and approves both texts.
  2. Search each page for `[`. No match may remain.
  3. Delete the element with the class `legal-draft` in `LegalPage.astro`.
  4. Delete the `.legal-draft` and `.legal-draft-title` rules and the
     comment above them in `legal.css`.
  5. In `ts/site/README.md`, delete the rule on the draft note and the
     rule on placeholders. Then describe `/terms/` and `/privacy/` as the
     approved texts.
- **Impact if no action:** The public site shows draft legal texts with
  open placeholders.
- **Needs:** The completed details and the approval of the user.
- **Done when:** Both pages have no placeholder and no draft note, the
  user approved both texts, and the site README describes them as approved.
- **Related:** `site-and-app-publication.md` waits for this task.
