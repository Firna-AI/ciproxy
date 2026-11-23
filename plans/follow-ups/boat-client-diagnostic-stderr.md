# The Boat client rejects warnings of successful commands

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-21, from `calummoore/ci-proxy-load-balancer`, live
  run 6, item 1 of its report.
- **Location:** `futex-ai/sandbox`, `crates/sandbox-boat/src/runtime/client.rs`
  line 77 at revision `ac412fc`, which ciproxy pins in `Cargo.toml`.
- **Reason:** The change is in `futex-ai/sandbox`, where the user approves
  each merge, and it waits until after the production launch.

- **Terms:**
  - **Helper command:** a fixed command that the pinned Boat client runs in
    a sandbox for a safe file transfer, such as the registration of a
    runner.
  - **stderr:** the stream on which a command prints its warnings and
    errors.
  - **Boat's controller:** Boat's agent in a sandbox, which runs the
    commands that the Boat client sends, such as the helper commands.
  - **Production launch:** the first start of the production environment
    of ciproxy. `plans/follow-ups/production-launch.md` holds that work.
- **Background:**
  - Before a Boat runner starts, ciproxy transfers its registration into
    the sandbox with helper commands.
  - The pinned client checks each helper reply: its status, its exit code,
    its output, its size, and its SHA-256. It also rejects each reply that
    has any text on stderr.
- **Problem:**
  - On 20 November 2026, each command on the Boat snapshot
    `2026-11-20-b751ef5f` printed this warning on stderr:

    ```text
    setlocale: LC_ALL: cannot change locale (en_US.UTF-8)
    ```

  - The client rejected the successful helper replies, so the transfer
    stopped and no runner started.
  - The Boat `large` job, the overflow smoke, and the spend-limit smoke
    failed. The idle-delete smoke passed.
  - PR #96, "fix(image): keep Boat controller locale available on
    restore", merged on 21 November 2026. It fixed the warning at its
    source.
  - Since PR #96, the Boat build writes standalone locale data outside the
    directory that Boat regenerates at a restore. The image selects that
    data only for the account of Boat's controller.
  - The client policy stayed strict. So the client still rejects each
    helper reply with text on stderr.
- **Impact if no action:** Any new harmless warning on Boat, from Boat's
  base system or from the image, stops every Boat start. Automatic retries
  then start more paid sandboxes.
- **Root question:** How does a harmless warning stay from blocking a
  successful helper command?
- **Options:**
  - **A.** Keep the strict client, and fix each warning at its source.
    Effort: Small for each warning. PR #96 did this for the locale warning.
  - **B.** Accept a helper reply with text on stderr when its status, exit
    code, output, size, and SHA-256 checks pass. Log the stderr text as a
    warning. Change `futex-ai/sandbox`, and update the pin in ciproxy.
    Effort: Medium.
- **Left out:** An accept of failed commands could accept an unsafe
  transfer.
- **Wider fix:** B. It separates the success of a command from its
  diagnostic output.
- **Previous examples:** `futex-ai/sandbox` #14 fixed a reply with a home
  relative path on 19 November 2026.
- **Recommendation:** B. A warning must not reject a transfer that every
  other check proves.
- **Action:** ask the user after the production launch, because the change
  is in `futex-ai/sandbox`, and the user approves each merge there. On 22
  November 2026, the orchestrator agent of
  `conductor/ciproxy-boat-start-ready-deadline` decided under the user's
  delegation that the change waits until after the production launch.
