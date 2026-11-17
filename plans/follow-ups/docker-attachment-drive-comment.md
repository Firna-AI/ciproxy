# Docker drive comment describes unused selection

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-17, from the guest and release review check.
- **Location:** `crates/ciproxy-protocol/src/cache.rs`,
  `crates/ciproxy-guest/src/handoff.rs`, and `image/docker-attach.sh`.
- **Reason:** The comment and guest disk selection predate this branch.

**Source:** Review. **Category:** Docs or spec.

**Terms:**

- **Docker attachment:** The host's description of the separate Docker
  disk supplied to a job VM.

**Background:** `DockerDiskAttachment` carries `free_bytes`, `drive_id`,
and `access`. The host selects the disk and its persistence policy.
The guest passes the free-byte requirement to its disk setup script.

**Problem:** The `drive_id` comment says the identifier finds the disk.
The guest does not read that field or `access`.
The script mounts `/dev/vdb` and uses only `free_bytes`.
The fields still form part of the JSON contract.

**Impact if no action:** A reader can assume the guest selects a disk
from `drive_id`, although it uses a fixed device.

**Root question:** How should attachment documentation describe the
guest's current use of its metadata?

**Options:**

- **A.** Correct the field comment to describe host-supplied metadata.
  Keep the current wire fields and guest behavior. Effort: Small.
- **B.** Audit consumers and compatibility before removing unused fields.
  This changes the JSON contract and needs a separate decision.
  Effort: Medium.

**Left out:** Teaching the guest to select a device from these fields
adds behavior that no current requirement needs.
The useful approaches correct the description or change the contract.

**Wider fix:** The documentation skill already requires comments to
describe current code. No new rule or gate is needed.

**Previous examples:** None recorded.

**Recommendation:** A. It corrects the false description while preserving
the current contract.

**Action:** Fix after merge, apply A, because this comment predates the
guest command cleanup and is outside its selected scope.
