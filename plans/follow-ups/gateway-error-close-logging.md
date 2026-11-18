# Hosted logs hide refused agent connections

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-18, from `calummoore/ci-proxy-load-balancer`, the
  options check of the live run 5 items.
- **Location:** `crates/ciproxy-gateway/src/transport.rs`, function
  `close`. `crates/ciproxy-gateway/src/logging.rs`, function
  `default_targets`.
- **Reason:** A log line for each refused connection needs a rate rule
  against floods, and the production edge limits change the same path.

- **Background:**
  - The gateway ends a refused install token, a refused machine credential,
    and a passed limit with an error close. The close frame has the
    WebSocket code 1008 and a fixed reason, as
    `docs/protocol/agent-connection.md` says.
  - `close` in `crates/ciproxy-gateway/src/transport.rs` logs each error
    close at the debug level.
  - Without `RUST_LOG`, `default_targets` keeps the default maximum level
    of `tracing_subscriber`, which drops debug events.
- **Problem:** The hosted logs show no refused install token, no refused
  machine credential, and no passed limit.
- **Impact if no action:** Support cannot see why an agent connection
  failed, for example after an expired install token.
- **Root question:** How does support see why the gateway refused an agent
  connection?
- **Options:**
  - **A.** Log each error close at the info level with its fixed reason and
    no credential. An unauthenticated client can then fill the logs.
    Effort: Small.
  - **B.** Count the error closes by reason in a metric, and log at the info
    level at most one line per reason in each interval. Effort: Medium.
  - **C.** Show refused attempts of an install token in the dashboard. This
    needs a store, an API, and a screen. Effort: Large.
- **Left out:** A log line with the credential breaks the rule that no log
  holds a secret.
- **Wider fix:** B. It covers each reason of an error close, also a new
  one, with a bounded log volume.
- **Previous examples:** None found.
- **Related:**
  - `plans/follow-ups/install-failure-messages.md`: same failure seen from
    the agent.
  - `plans/follow-ups/gateway-edge-connection-limits.md`: same code. It adds
    the production limits for each client address.
- **Recommendation:** B. Support sees each kind of refusal, and a flood
  cannot fill the logs.
- **Action:** fix after merge, apply B, because a log line for each refused
  connection needs a rate rule against floods, and the production edge
  limits change the same path.
