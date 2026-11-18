# Install gives up after one failed connect

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-18, from `calummoore/ci-proxy-load-balancer`, the
  options check of the live run 5 items.
- **Location:** `crates/ciproxy-agent/src/installation.rs`, method
  `Installer::install`. `crates/ciproxy-agent/src/connection.rs`, method
  `WebSocketConnection::connect`. `crates/ciproxy-agent/src/gateway.rs`,
  method `ReconnectingGateway::reconnect`.
- **Reason:** The failure needs a passing network fault or a full
  handshake limit, and a second run of the command recovers.

- **Terms:**
  - **Handshake limit:** each gateway instance has at most eight
    handshakes in progress. It answers each other upgrade with HTTP 503, as
    `docs/protocol/agent-connection.md` says.
- **Background:**
  - `Installer::install` connects to the gateway once, with a time limit of
    20 seconds. Each failure ends the install with exit 1.
  - The running agent retries its connection with a growing delay, in
    `ReconnectingGateway::reconnect`.
  - `docs/protocol/agent-connection.md` lets an agent try again later after
    a 503 of the handshake limit.
- **Problem:**
  - The install stops at the first failed connect, also for a 503 of the
    handshake limit or a short network fault. The customer must run the
    command again.
  - On 18 November 2026, the first customer install of agent 0.1.3 failed
    before the gateway got a Register message, and a retry passed. Its
    cause is not known, so this item does not claim that a retry would
    have fixed it.
- **Impact if no action:** A passing fault fails an install that a short
  retry would finish.
- **Root question:** How does the install command get past a passing fault
  before Register?
- **Options:**
  - **A.** Retry the connect and the upgrade with a growing delay, for at
    most about one minute. Retry a 503 and a network failure, not a refused
    install token. Print one line for each retry. Effort: Small.
  - **B.** Keep one attempt. Print a text that says to run the command
    again for a 503 or a network failure.
    `plans/follow-ups/install-failure-messages.md` gives that text. Effort:
    Small.
- **Left out:** A retry of the whole command in the install script repeats
  the host preparation and also retries a refused install token. A is
  better on every point. No third place exists: the retry is in the agent,
  in the script, or nowhere.
- **Wider fix:** A, when the install and the running agent share one retry
  helper.
- **Previous examples:** None found.
- **Related:** `plans/follow-ups/install-failure-messages.md`: same code.
  It names each install failure, and B uses its text.
- **Recommendation:** A. The protocol allows the retry, and the customer
  needs no second run.
- **Action:** fix after merge, apply A, because the failure needs a passing
  fault or a full handshake limit, and a second run of the command
  recovers.
