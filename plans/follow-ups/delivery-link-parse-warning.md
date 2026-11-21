# A broken Link header ends recovery silently

- **Severity:** Low. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, a problem that the check of
  finding 1006 found.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-github/src/redelivery.rs`,
  `HttpDeliveries::list`, its parse of the `link` header.
- **Reason:** The parse matches the documented format of GitHub today, so
  only a format change of GitHub meets this problem.

- **Terms:**
  - **Webhook recovery:** `RecoverDeliveries::retry_failed`, which reads the
    webhook delivery history of the App and asks GitHub to redeliver each
    failed event. It runs once a minute.
  - **`Link` header:** the HTTP header in which GitHub gives the cursor of
    the next history page, in the form `<…?cursor=…>; rel="next"`.
- **Background:**
  - The webhook recovery must read every page of the delivery history.
    `HttpDeliveries::list` reads one page, and takes the cursor of the next
    page from the `Link` header.
  - Each failure of that parse gives `None`. The failures are:
    - a header that is not valid text
    - a `rel="next"` entry without `<…>`
    - a URL that does not parse
    - a URL with no `cursor` query
  - `None` means the last page. So the scan ends after that page, with no
    log line.
  - Events on the later pages then get no redelivery from this pass, or
    from any later pass while the format stays.
  - `crates/ciproxy-control-plane/github-workers.md` says that a lost event
    is not safe.
  - The always-on rules of this repository say: "When input is
    recoverable, warn and continue."
  - On 17 November 2026 the final review chose a test of the paging through
    a local HTTP peer with two linked pages. That test does not change the
    parse.
- **Problem:** A `Link` header with a `rel="next"` entry that does not
  parse ends the scan as if it were the last page, and nothing logs it.
- **Impact if no action:** After a format change of GitHub, the recovery
  stops after the first page with no warning. Failed events on later pages
  then wait for no redelivery, and nobody learns why.
- **Root question:** How does the webhook recovery tell an operator when it
  cannot read the cursor of the next page?
- **Options:**
  - **A.** Log a warning when a `rel="next"` entry exists but gives no
    cursor, and end the scan as now. Cover the case in the test of the
    webhook recovery that runs over a local HTTP peer. Effort: Small.
  - **B.** A, and follow the full `next` URL when the cursor is missing but
    the URL parses on the GitHub API origin. Cost: it changes working
    paging code, and it must refuse a URL on another origin.
    Effort: Small.
  - **C.** Treat a broken `rel="next"` entry as an error of the pass. Cost:
    the pass then redelivers no event at all, because it reads every page
    before it redelivers. Effort: Small.
- **Left out:** None. The three options cover a warning, a fallback, and an
  error.
- **Wider fix:** None. The rule "When input is recoverable, warn and
  continue" is in force. A follows it.
- **Previous examples:** None found.
- **Recommendation:** A. It follows the rule with one log line, and it keeps
  the redeliveries of the pages that the scan read.
- **Action:** fix after merge, apply A, because the parse matches the
  format of GitHub today.
