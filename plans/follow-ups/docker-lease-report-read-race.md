# A lease removed mid-read fails Docker reads

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-16, from `conductor/m26-docker-disks`, the independent
  availability check of the Docker disks audit of Milestone 26.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/docker_files.rs:45-66`, with the
  read at `:59`, `crates/ciproxy-agent/src/docker.rs:154-172` and
  `:174-190`, and `crates/ciproxy-agent/tests/docker/lease_tests.rs`.
- **Reason:** The fix is agent code that the branch which found the race
  does not own, and nothing in that branch depends on it.

- **Terms:**
  - **Lease file:** one file under `docker-leases` in the agent data
    directory. It records the Docker disk reservation of one job VM.
    `docker_files::save` writes it, at `docker_files.rs:68-75`.
  - **Pending file:** the `.pending` file of an atomic write, named by
    `private_write_temporary` at
    `crates/ciproxy-agent/src/persistence.rs:72-75`.
  - **Control worker:** the agent task that runs control messages and guest
    events, `control_worker` at
    `crates/ciproxy-agent/src/runtime_worker.rs:29-87`.
- **Background:**
  - `docker_files::leases`, at `docker_files.rs:45-66`, lists the lease
    directory. It skips pending files, at `:56-58`. It reads each other
    listed file at `:59`, and it decodes the file at `:62`.
  - A missing directory gives an empty list, at `:49`. But when a listed
    file is gone before the read, `fs::read` returns NotFound, and `?` fails
    the whole read.
  - `release()`, at `crates/ciproxy-agent/src/docker.rs:154-172`, is the
    only code that removes a lease file. It removes the Docker disk of the
    job first, then the lease file, at `:156-165`. So a lease that is gone
    holds no disk.
  - A job end calls `release()` through `retire()`, at
    `crates/ciproxy-agent/src/retirement.rs:64-74`, for the guest event
    `RunnerExited`, at `crates/ciproxy-agent/src/observations.rs:185-190`.
    A failed preparation calls it too, at `docker.rs:148`.
  - Guest events run in the control worker, which
    `crates/ciproxy-agent/src/runtime.rs:64-70` spawns as its own task. The
    report loop runs in the main task, in the `tokio::select!` at
    `runtime.rs:87-90`, through
    `crates/ciproxy-agent/src/runtime_reports.rs:42-61`.
  - The agent runtime is multi-threaded: `#[tokio::main]` at
    `crates/ciproxy-agent/src/main.rs:181`, with the tokio `full` features.
    So the two tasks can run at the same time.
  - `machine_report()` reads the Docker figures before it takes the
    controller state lock, in `crates/ciproxy-agent/src/reporting.rs`.
    `retire()` takes no shared lease-read lock. No lock orders the two.
  - So a job end can remove a lease between the listing and the read of a
    report. `LinuxDockers::report()`, at `docker.rs:174-190`, then fails.
  - The report logs the failure and marks its Docker figures unavailable.
    The page shows "Unknown", and the hosted capacity becomes unknown.
    The scaler starts no new runner there until a measured report arrives.
  - Every lease reader calls `leases()`: `reserved()` at `docker.rs:71-75`,
    `prepare()` at `:88`, `report()` at `:176`, `held()` and `unused()` at
    `docker_files.rs:77-94`, and Docker recovery at
    `crates/ciproxy-agent/src/docker_recovery.rs:52`.
  - The readers that run in the control worker run one at a time with
    `release()`. These include the capacity check before each runner start,
    at `crates/ciproxy-agent/src/launches.rs:53-63`. So the race reaches
    mainly the machine report.
  - `bases()`, at `docker_files.rs:96-119`, already skips a repository disk
    that is gone, at `:106-110`. `load()`, at `:34-43`, already reads a
    missing lease as none.
  - The tests in `crates/ciproxy-agent/tests/docker/lease_tests.rs` use a
    real temporary directory, through `TemporaryLayout` at
    `crates/ciproxy-agent/tests/docker/fixtures.rs:29-39`.
    `cargo test --package ciproxy-agent` runs them.
  - Two of these tests prove the pending-file filter:
    `complete_pending_lease_does_not_hold_capacity` and
    `partial_pending_lease_does_not_block_reporting`. No test proves that a
    lease that does not decode still fails the read.
  - Neither option changes what the report sends after a failed read.
    `docs/protocol/docker-disks.md#unavailable-figures` states that rule.
  - The per-file read came with commit `1722b62` of 2026-11-06.
- **Problem:** A lease that a job end removes after the listing fails the
  whole Docker read with NotFound. The released lease holds no disk, so the
  read should count the leases that remain.
- **Impact if no action:** Rarely, the page shows unknown Docker figures
  and capacity. New runner starts wait for the next measured report,
  although the removed lease holds no disk.
- **Root question:** How does the agent read its Docker leases while a job
  end removes one of them?
- **Options:**
  - **A.** In `leases()`, treat a NotFound from the read of a listed lease
    as a lease that is gone, and skip it.
    - Keep each other I/O error and each decode error as a failure. Keep the
      pending-file filter.
    - Capacity stays safe, because a lease that is gone holds no disk.
    - Add a regression to `crates/ciproxy-agent/tests/docker/lease_tests.rs`.
      A listed lease that is gone at the read must not stop `report()` from
      counting the other leases.
    - A dangling symbolic link with the name of a lease gives that NotFound
      with no timing, for example.
    - Add the missing negative case: a lease that does not decode still
      fails `report()`.
    - Use no sleep, retry, or thread timing in the tests.
    - Effort: Small.
  - **B.** Order lease changes and lease reads with one lock. Every lease
    writer, `release()`, every lease reader, and Docker recovery take it.
    - The lock must not cover the disk preparation, which awaits.
    - Cost: a new lock that `LinuxDockers` and the controller share, and a
      slower report while it waits for a lease change.
    - Gain: each reader sees one consistent set of leases.
    - Effort: Medium.
- **Left out:**
  - A retry of the read after NotFound, because a lease that is gone needs
    no second read, and a retry adds time.
  - Skipping every read error, because a lasting I/O or decode error must
    still fail the read.
  - A lease list in agent memory as the source for readers, because it
    changes the durable record that recovery reads. B gives the same
    consistency with less change.
- **Wider fix:** A. Every lease reader calls `leases()`, so one change
  covers each reader. Nobody traced the other directory readers of the
  agent for the same pattern.
- **Previous examples:** None found as a defect. `bases()` and `load()` in
  `docker_files.rs` already treat a file that is gone as absent.
- **Related:** `docs/protocol/docker-disks.md#unavailable-figures`: the same
  reader and report path. This race is one cause of unavailable figures.
  The availability rule already applies. The direct race fix remains
  separate.
- **Recommendation:** A. It fixes the race in the one shared reader, and it
  keeps every real error.
- **Action:** fix after merge, apply A, because the fix is agent code
  outside the branch that found the race, and nothing in that branch
  depends on it.
