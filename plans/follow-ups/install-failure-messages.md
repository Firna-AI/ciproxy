# Install failures print only an internal error

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-18, from `calummoore/ci-proxy-load-balancer`, live
  run 5, item 5 of its report.
- **Location:** `crates/ciproxy-agent/src/installation.rs`, method
  `Installer::install`. `crates/ciproxy-agent/src/connection.rs`, method
  `WebSocketConnection::receive`. `crates/ciproxy-agent/src/error.rs`, enum
  `Error`. `crates/ciproxy-agent/src/main.rs`, function `main`.
  `crates/ciproxy-agent/src/host_preparation.rs`, functions `stage` and
  `stage_error`.
- **Reason:** The fix gives the customer new facts and new actions in the
  error texts of the install command, and no protocol doc states them.

- **Terms:**
  - **Install token:** the token in the install command from the
    dashboard. It works for one hour. The agent sends it in its Register
    message.
  - **Error close:** a close frame from the gateway with the WebSocket code
    1008 and a fixed reason, as `docs/protocol/agent-connection.md` says.
  - **Internal error:** the fallback variant `Error::Internal` of the
    agent. It keeps its cause chain in memory and prints only
    `[agent] internal error`.
  - **Stage error:** `stage_error` in
    `crates/ciproxy-agent/src/host_preparation.rs`. It turns any failure of
    one host preparation stage into a text with the stage and its repair.
- **Background:**
  - `ciproxy-agent install --token <token>` prepares the host, loads and
    saves the machine state, and reads the machine report. Then it
    connects to the gateway, sends Register, waits for Registered, and
    installs the service.
  - `docs/protocol/agent-connection.md` says that the error close with the
    reason `[gateway] authentication failed` rejects an install token or a
    machine credential.
  - The same doc says that while eight handshakes are in progress, the
    gateway answers each new upgrade with HTTP 503.
  - The copy rule of the product says that an error names what failed and
    gives one recovery action when one exists.
  - The error contract of the pinned `internal-error` dependency, in its
    `docs/protocol/error-contracts.md`, says that a top-level binary logs
    the full internal failure. The text for the user holds no cause chain.
  - Live run 5 reported the unexplained first install as its item 5, with
    three options: capture diagnostics in the next live install, accept
    the retry, or add stage diagnostics to the agent. This item replaces
    it.
- **Problem:**
  - `WebSocketConnection::receive` turns an error close that arrives before
    Hello into `ConnectionError::Disconnected`. Register is not Hello.
  - `Installer::install` wraps that error with
    `.context("receive registration")`, so it becomes an internal error. A
    refused install token prints only `[agent] internal error`.
  - A failed connect, a TLS failure, an HTTP 503 of the handshake limit,
    and the two time-outs of 20 seconds print the same line.
  - `main` logs only the display text of the error. `stage_error` and the
    warning `control command failed` do the same. So the cause chain goes
    nowhere, and neither the customer nor ciproxy can see the failed step.
  - On 18 November 2026, the first customer install of agent 0.1.3 failed
    with exit 1 before the gateway got a Register message. Its cause is
    not known. A retry with a new install command passed.
  - A run with a revoked install token printed only this line, without
    its time stamp and target:

    ```text
    {"level":"ERROR","fields":{"message":"agent command failed","error":"[agent] internal error"}}
    ```

- **Impact if no action:** A customer who runs an install command after
  its hour sees an internal error and gets no next step. A network or TLS
  problem on the host looks the same, so support cannot tell the cases
  apart.
- **Root question:** How does a person who runs the install command learn
  what failed and what to do next?
- **Options:**
  - **A.** Give the expected install failures typed errors. A refused
    install token says that the token was refused and to create a new
    install command in the dashboard. A failed connect or handshake names
    the control plane host and says to check outbound HTTPS access.
    Effort: Small.
  - **B.** Apply A. Also write the cause chain of each internal error of a
    command to a log file on the machine that only root can read. The
    printed text names that file. The chain must drop URLs first, because
    a download error holds the signed image URL. The file needs a
    retention rule. Effort: Medium.
  - **C.** Give each install step a typed failure that names the step and
    one action, as the stage error does for host preparation. The steps
    are: load and save the machine state, read the machine report, reach
    the control plane, register, and install the service. A refused
    install token keeps the text of A. The connect step tells a missing
    answer apart from an HTTP error answer, such as a 503 that asks for a
    later retry. An unforeseen failure then still names its step. It adds
    about five texts for the user to review. Effort: Medium.
  - **D.** Apply A. Also send the full internal failure of each command to
    the systemd journal, not to a new file. The terminal keeps the short
    text. It needs a new dependency, such as `tracing-journald`. Members of
    the `adm` and `systemd-journal` groups can read the journal. The chain
    must drop URLs first, as in B. Effort: Medium.
- **Left out:**
  - Diagnostics only in the next live install, or acceptance of the retry,
    leave each customer with the internal error.
  - The cause chain on the terminal breaks the copy rule and the redaction
    rule of the error contract. Both rules stay.
  - The hosted gateway logs fail the goal. The person who runs the command
    cannot read them, and the gateway sees no DNS, TCP, or TLS failure.
- **Wider fix:** C. A new failure inside a step still names its step. B and
  D record the cause, but the terminal still prints the generic line.
- **Previous examples:** Host preparation met this class before. Commit
  `43e4f813` added `stage` and `stage_error`. `execute` in
  `crates/ciproxy-agent/src/application.rs` uses `stage_error` for the
  control directory and the identity state.
- **Related:**
  - `plans/follow-ups/install-connect-retry.md`: same code. The install
    does not retry a failed connect or a 503, which is a separate root
    question.
  - `plans/follow-ups/gateway-error-close-logging.md`: same failure seen
    from the gateway. The hosted logs show no refused install token.
- **Recommendation:** C, changed from A. It names the step of every install
  failure, also an unforeseen one like the first customer install of live
  run 5, for a little more work than A. D best meets the logging rule of
  the error contract, but it adds a dependency and needs URL redaction.
- **Action:** ask the user, because the new error texts add facts and
  actions that no protocol doc states, and the user reviews each new
  user-visible text of this kind.
