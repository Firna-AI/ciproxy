# Production has no smoke test yet

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the launch milestone of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** None.
- **Reason:** The smoke test needs the deployed production environment,
  which exists only after the launch pull request merges.

- **Background:**
  - The main preview passed the live checks of the product before the
    launch: a machine install, jobs, warm runners, slots, drain, restart,
    removal, Boat jobs, and a Stripe test-mode Checkout.
  - Production uses its own App, its own Stripe live mode, its own job
    image release, and full agent releases only. So the preview results do
    not prove the production setup.
- **Task:**
  1. Sign in at `https://app.ciproxy.dev`. Install the production App on a
     test organization with a private repository.
  2. Register a KVM test machine with the install command of the
     dashboard. Run a first job on it.
  3. Connect a Boat test account. Make the machine busy, and show that a
     job overflows to Boat after the wait.
  4. Run one live Checkout that the user approves, with a real card. The
     user decides whether to cancel and refund it after the check.
  5. Save only safe output. Record no cookie, key, token, or card detail.
- **Impact if no action:** A wrong production value, such as a key, a
  host, or an image path, reaches the first customer.
- **Needs:** `production-launch.md` done. A KVM test machine, a Boat
  account key, and the user's approval and card for the live Checkout.
- **Done when:** A first job, an overflow job, and a live Checkout pass on
  production.
- **Related:** `production-launch.md` comes first.
  `site-and-app-publication.md` waits for this task.
