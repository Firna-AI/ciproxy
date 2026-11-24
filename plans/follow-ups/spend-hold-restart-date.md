# Spend notices can name a false restart date

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-24, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, review finding 12.
- **Location:** `crates/ciproxy-overflow/src/view.rs:172`,
  `crates/ciproxy-api/src/handlers/jobs_live.rs:208`,
  `ts/app/src/view/jobs-hold-facts.ts:55`, `ts/app/src/view/overflow.ts:182`,
  and `docs/protocol/overflow-spend.md:99`.
- **Reason:** The fix replaces the protocol's promise that starts resume next
  month with a conditional date, and only the user changes a requirement.

- **Source:** Review.
- **Category:** Product bug.
- **Terms:**
  - **Spend limit:** the most that a customer lets ciproxy commit for one
    provider in a UTC calendar month.
  - **Start bound:** the cost that ciproxy reserves for a sandbox's full
    lifetime before it sends the create request. A bound can touch two UTC
    months.
- **Background:**
  - Since review finding 1, ciproxy holds a provider when its limit cannot
    cover the bound of the next start, not only when spend reached the
    limit.
  - Jobs and Settings give the first day of the next UTC month as the date
    when the provider takes jobs again, as
    `docs/protocol/overflow-spend.md:102` promises.
  - Each label hold uses the label's own size. The provider notice uses the
    provider's cheapest size.
  - One Boat start bound is about $0.50 for `large` and about $1.27 for
    `xlarge`, with the start fee.
- **Problem:** The next month can still lack room for one start. With a $1
  limit and a $1.27 bound, an empty month still holds the provider, but the
  notice names the first of next month. A label whose own size never fits
  still gets the provider's date.
- **Impact if no action:** A customer with an unusually low limit waits for a
  date when the jobs still do not start.
- **Root question:** What should each spend notice say about when its
  provider takes jobs again, when the start of the next month alone does
  not end the hold for its size?
- **Options:**
  - **A.** Show the UTC reset date as budget information only, and say that
    starts resume when there is enough budget. Change the mockups and the
    protocol. Effort: Medium.
  - **B.** Send one budget reason for each provider with an optional restart
    estimate. Effort: Medium.
  - **C.** Show the counted spend and the next start's cost by month. Effort:
    Large.
  - **D.** The admission check returns a restart condition for each hold: the
    provider hold for its cheapest size, and each label hold for its own
    size. The date is the first of the next month only when that month's
    counted spend plus one bound fits the limit. Otherwise the notice says
    that one start costs more than the limit, and keeps "Raise the limit".
    Jobs shows one notice for each provider, hold, and condition. Change the
    mockups first, then the protocol, the spend guide, Settings, Machines,
    and Jobs. Effort: Medium.
  - **E.** D, and name the cost of one start when the limit is below it.
    Effort: Medium.
- **Left out:** Admitting a start on this month's cost alone, because it
  breaks the full-lifetime bound. Going back to "counted spend reached the
  limit", because the provider then takes no job and no notice says why. A
  warning or a rejection at the limit form, because prices and label sizes
  change after the limit is saved.
- **Wider fix:** D. No notice can give a date that admission does not
  support.
- **Previous examples:** `6450c3e8` added the bound for each month a start
  touches. `c4e7fac8` recorded the approval to show the month and the
  restart date.
- **Related:** When the pending bounds of running sandboxes fill the limit,
  the hold often ends within minutes, when the sandboxes report runtime, but
  the notice still names next month. D's condition covers that case too.
- **Recommendation:** D. It removes every false date and keeps the approved
  date where the budget supports it.
- **Action:** ask the user, because D replaces the protocol's promise that
  starts resume next month.
