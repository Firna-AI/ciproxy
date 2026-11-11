# Guest initialization failure leaves Firecracker running

- **Severity:** High. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-08, from the independent review of
  `conductor/fix-agent-live-defects-unconfigured-stop-xfs-first-install`.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `image/ciproxy-guest.service:11`.
- **Reason:** The image service belongs to separate image work, and the
  agent's proven-unconfigured retirement fix works without this change.

- **Terms:**
  - **Guest:** The ciproxy program inside a job VM.
  - **Firecracker:** The host process that runs that VM.
  - **Initialization hook:** The systemd `ExecStopPost` command that runs
    after the guest service ends, including an early setup failure.
- **Background:** The guest program can fail before its supervisor starts.
  The image service then runs its initialization hook.
  The normal guest shutdown path uses a reset to end x86 Firecracker.
  The agent supplies the `reboot=k` kernel argument for that reset.
- **Problem:** The initialization hook still calls `/sbin/poweroff`.
  x86 Firecracker has no ACPI power control, so poweroff leaves its host
  process alive. A failure before hello also supplies no proof that the
  guest never received a registration.
- **Impact if no action:** An early guest setup failure can leave its VM
  and reserved capacity in place until host cleanup ends it.
- **Root question:** How does guest initialization failure end its VM?
- **Options:**
  - **A.** Use the Firecracker reset path in the shared image hook. Keep
    the guest's normal shutdown path. Publish the changed image and prove
    initialization failure on KVM. Effort: Small.
  - **B.** Move all initialization cleanup into the guest power boundary.
    Remove the separate image hook after it covers every early failure.
    This needs a larger lifecycle change and image release. Effort: Medium.
- **Left out:** A manual host stop does not provide automatic cleanup.
  Ending every guest without a hello can stop an existing running job.
- **Wider fix:** A. The shared image hook handles each initialization
  failure with the correct shutdown operation.
- **Previous examples:** The guest's normal power adapter had the same
  wrong poweroff operation. `crates/ciproxy-guest/src/power.rs` now uses
  `/sbin/reboot` for Firecracker and leaves Boat shutdown to its wrapper.
- **Recommendation:** A. The existing shared hook already owns this exit
  path and needs only the correct Firecracker shutdown operation.
- **Action:** fix after merge, apply A, because the image owner must build
  and publish this change outside the agent defect branch.

The [Firecracker shutdown contract][shutdown] states that x86 poweroff
leaves Firecracker alive, while reboot with `reboot=k` ends its process.
The live check must show that an early initialization failure ends the
Firecracker process and returns VM capacity with the published image.
It must also show that the image hook preserves the guest's normal reset.

[shutdown]: https://github.com/firecracker-microvm/firecracker/blob/v1.17.0/FAQ.md#how-can-i-gracefully-reboot-the-guest-how-can-i-gracefully-poweroff-the-guest
