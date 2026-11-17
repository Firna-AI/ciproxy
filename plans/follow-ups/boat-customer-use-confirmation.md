# Boat has not confirmed customer runner use

- **Severity:** Medium. **Effort:** Small. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the launch milestone of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** None.
- **Reason:** The user sends the request to Boat, and the answer comes
  from outside the repository.

- **Terms:**
  - **Boat:** the overflow provider. ciproxy rents Boat sandboxes when the
    machines of a customer are busy.
- **Background:**
  - A customer connects its own Boat account in the dashboard with its
    Boat key. ciproxy then starts Boat sandboxes with that key. Each
    sandbox runs a GitHub Actions runner for the jobs of that customer.
  - On 4 November 2026 the orchestrator read the Boat terms. Its call was
    that the terms allow this use by inference only.
- **Task:**
  1. The user asks Boat to confirm in writing that a customer may run
     ciproxy runners on its own Boat account.
  2. Record the answer and its date in this item.
  3. If Boat refuses or sets conditions, raise a decision on the overflow
     feature before the site is published.
- **Impact if no action:** Boat could later restrict the accounts of
  customers that run ciproxy runners, and their overflow jobs would stop.
- **Needs:** The user's contact at Boat.
- **Done when:** Boat confirms the use in writing, or the user decides the
  change that a refusal needs.
- **Related:** `site-and-app-publication.md` waits for this task.
