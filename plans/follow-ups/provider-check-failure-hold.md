# A provider outage reads as current for minutes

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-24, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the checks of review
  findings 9 and 11.
- **Location:** `crates/ciproxy-overflow/src/service/refresh.rs:57`,
  `crates/ciproxy-overflow/src/service/usage.rs:43`, and
  `crates/ciproxy-scaler/src/overflow.rs:34`.
- **Reason:** The fix adds a new hold reason with dashboard copy and a
  mockup, which is wider than the review fixes of the branch.

- **Source:** Review.
- **Category:** Product bug.
- **Terms:**
  - **Provider check:** the refresh of a provider's account facts that runs
    between two scaler passes.
  - **Unsent start:** a saved request for a provider runner that ciproxy has
    not yet sent to the provider.
- **Background:**
  - The refresh reads the provider account only once in five minutes, and
    it ignores failed usage reads.
  - Each start preparation reads the account on every attempt.
  - Review finding 9 gave the scaler a check state: in progress, current, or
    failed. A failed check sends a waiting job to the next place of its
    label order.
  - A second, older case: when the overflow wait is "At once", the pass
    before the refresh can stop warm runners for a job whose order puts own
    machines first, and the pass after the refresh then starts that job on
    the provider.
- **Problem:** For up to five minutes of a provider outage, the check reads
  as current. Each new provider-first job then gets an unsent start that its
  preparation cannot send, and the job does not move on.
- **Impact if no action:** During the first minutes of an outage, jobs can
  wait for the provider while own machines are free.
- **Root question:** How does a failed provider call become a fact that
  routing sees at once?
- **Options:**
  - **A.** Let a failed start preparation mark the provider check as failed
    until the next successful refresh. Effort: Medium.
  - **B.** Save a failed provider call as a start hold in the account state,
    and show it as a hold reason in the dashboard, with mockup and protocol
    changes. Effort: Medium.
  - **C.** Read the account on every refresh. This adds provider calls on
    each pass. Effort: Small.
- **Left out:** A time limit on an unsent start, because it adds a wait
  between places, against the decision of no extra wait.
- **Wider fix:** B. Every decision path reads a saved hold.
- **Previous examples:** `boat-uncertain-create-holds-job.md` records a
  related wait after a create may have reached Boat.
- **Recommendation:** A. It uses the check state of finding 9 and needs no
  new copy.
- **Action:** fix after merge, apply A, because the case is rare and the
  branch already fixed the failed-refresh case.
