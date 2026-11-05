# Unknown costs lack savings suppression coverage

- **Severity:** Medium. **Effort:** Small. **Confidence:** High. **Harder
  later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-05, from the finding check for PR #19.
- **Location:** `crates/ciproxy-metering/src/report/savings.rs:89` and
  `crates/ciproxy-metering/src/_tests_/savings_tests.rs`.
- **Reason:** The savings suppression code and its missing regression predate
  the calendar-day cost change.

- **Terms:**
  - **SavingsReport:** The workspace report with ciproxy costs, alternative
costs, and signed savings.
  - **CostGap:** A missing monthly cost fact that keeps the ciproxy total
unknown.
- **Background:** ciproxy compares the costs of a customer's jobs on its own
  machines and Boat with Blacksmith and GitHub-hosted runners. The savings
  report must hide both savings figures when a machine has no monthly cost. It
  must still show the alternative costs, Boat costs, and the known tier price,
  as `docs/protocol/metering.md` requires.
- **Problem:** The pure calculation uses `comparison.usd.zip(costs.total_usd)`
  to hide an incomplete savings figure. The current report tests cover a full
  known total, negative savings, and an unknown alternative. The cost tests
  cover missing machine and plan prices. No report test gives a missing ciproxy
  total and asserts that both savings figures are absent.
- **Impact if no action:** A later default or fallback for an unknown ciproxy
  total can show invented savings without failing a regression test.
- **Root question:** How should tests protect savings suppression when the
  ciproxy cost is unknown?
- **Options:**
  - **A.** Add one hand-worked case through `savings_report`. Supply a
registered machine without its price, known alternative price lists and runtime,
a known tier price, and a known Boat cost. Assert both exact `Saving` values
have `usd: None`. Assert the exact CostGap and the other visible known costs.
Prove an incorrect fallback total makes the test fail. Effort: Small.
  - **B.** Cover the same result through `PgStore` and `Metering::savings`. This
also reads persisted facts but needs database setup for a pure calculation rule.
Effort: Small.
- **Left out:** A private helper test uses the wrong entry point. A
  documentation test cannot protect the calculation. A unit mock that supplies
  final savings would test only the mock. These leave two useful test levels.
- **Wider fix:** None. The shared pure report already represents missing totals
  with Option. The missing regression is the problem.
- **Previous examples:** `missing_machine_costs_keep_other_costs_visible` tests
  missing machine costs at the cost level.
  `negative_savings_keep_their_sign_and_unknown_comparison_hides_lead` tests an
  unavailable alternative, not an unavailable ciproxy total.
- **Recommendation:** A. It checks the required report result at the lowest
  public calculation entry point.
- **Action:** fix after merge, apply A. This is a pre-existing test gap outside
  the calendar-day change.
