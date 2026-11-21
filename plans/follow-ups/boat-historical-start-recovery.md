# No command closes a proven unsent Boat start

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from plan item 121, option B of answer C.
- **Location:** `crates/ciproxy-control-plane/overflow.md`, the section
  "Cleanup and recovery", and the overflow recovery code in
  `crates/ciproxy-overflow`.
- **Reason:** The user chose a one-time repair for the only known case, and
  this command for later cases.

- **Terms:**
  - **Start record:** the saved record of one requested Boat sandbox.
- **Background:** The control plane saves a start before it calls the Boat
  client. #72 ends a fresh create that the client never sent as `Refused`.
  A start that an older client rejected before it sent anything stays
  `creating`, and the recovery keeps it pending while no sandbox exists.
  On 17 November 2026, the one such start in the preview was repaired once
  by hand, after a backup and a provider scan.
- **Problem:** No documented operator command closes a start that is proven
  never sent.
- **Impact if no action:** A later case blocks paid overflow checks until an
  operator edits the database by hand.
- **Root question:** How does an operator close a start that Boat proves it
  never created, safely and with an audit record?
- **Options:**
  - **A.** Add an operator command that checks the Boat inventory for the
    start, refuses when a sandbox may exist, and then ends the start as
    `Refused` through the same code path as #72, with runner cleanup and the
    rule that nothing is sent again. Document it in "Cleanup and recovery".
    Effort: Medium.
- **Left out:** Sending the create again, because it can make an unwanted
  paid sandbox. A manual database edit, because it bypasses the service
  rules.
- **Wider fix:** A.
- **Previous examples:** The repair of delivery
  `07a57f60-3ea3-4a04-8f07-1a1add1e17bf` on 17 November 2026.
- **Recommendation:** A.
- **Action:** fix after merge, apply A, as the user chose with answer 121 C.
