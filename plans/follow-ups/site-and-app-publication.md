# The site and the App are not public

- **Severity:** High. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the launch milestone of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** `deploy/launch.md`, the apex record step.
  `deploy/static-delivery.md`.
- **Reason:** Publication is the last launch step, after production and
  its smoke test exist.

- **Background:**
  - The site under `ts/site` holds the landing page, the pricing page, the
    terms, the privacy policy, and the docs.
  - Each deploy publishes the built site to the site buckets of its
    environment, as `deploy/static-delivery.md` says.
  - The production site becomes reachable at `https://ciproxy.dev` when
    the DNS-only apex record of `deploy/launch.md` exists.
  - The production GitHub App stays private until this step. Other
    organizations can install it only after it is public.
- **Task:**
  1. Check that these items are done: `legal-pages-approval.md`,
     `cost-claim-prices.md` for its check before the launch,
     `boat-customer-use-confirmation.md`, and `production-smoke-test.md`.
  2. Check that `https://ciproxy.dev` serves the current site.
  3. Make the production GitHub App public in its GitHub settings.
- **Impact if no action:** No customer can find the product or install
  the App.
- **Needs:** The four items above done. The owner access of the user to
  the App settings.
- **Done when:** `https://ciproxy.dev` serves the approved site, and the
  production App is public.
- **Related:** This task waits for the four items in its task list.
