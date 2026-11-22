# Small labels do not fit small machines

- **Severity:** Medium. **Effort:** Large. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the plan `plans/linux-arm64-own-machines.md`
  on the branch `calummoore/ci-proxy-on-raspberry-pi`.
- **Location:** `crates/ciproxy-protocol/src/resources.rs:35`,
  `crates/ciproxy-protocol/src/resources.rs:45`.
- **Reason:** On 22 November 2026 the user chose to leave the sizing for
  later.

- **Terms:**
  - **Slot:** the unit of machine capacity in
    `crates/ciproxy-protocol/src/resources.rs`: 2 hardware threads and 8 GB
    of memory.
  - **Host reserve:** the quarter of the threads and of the memory that a
    machine keeps for itself, as `docs/protocol/job-routing.md` says under
    "Slots".
- **Background:**
  - A machine has the smaller of two numbers: three quarters of its threads
    divided by 2, and three quarters of its memory divided by 8 GB. Each
    number is rounded down.
  - A job VM takes the larger of its vCPU divided by 2 and its memory
    divided by 8 GB, each rounded up. So every job VM takes at least 1 slot.
  - A label can be as small as 1 vCPU, 1 GB of memory, and 1 GB of disk, as
    `crates/ciproxy-api/src/handlers/label_views.rs:16` allows.
  - A machine with 8 GB of memory, such as a Raspberry Pi with 8 GB, keeps
    6 GB after the host reserve. That is less than 8 GB, so it has 0 slots.
    It runs no job, also a job of 1 GB.
  - A machine with 4 threads and 16 GB, such as a Raspberry Pi 5 with
    16 GB, has 1 slot. It runs one job at a time, also when each job needs
    only 1 GB.
  - The fixed ratio also wastes capacity on large machines. A label of
    2 vCPU and 16 GB takes 2 slots, so it holds 4 threads.
  - Slots are part of the agent wire protocol: `slot_indices` in
    `crates/ciproxy-protocol/src/runners.rs`, and `capacity` and `slots` in
    `crates/ciproxy-protocol/src/reports.rs`. They are also columns of the
    store. After launch, released agents use that protocol, so a change of
    the unit gets harder.
  - No doc or test measures whether a job VM of 1 GB boots the image and
    runs a job. Each job starts with Docker and containerd running, as
    `docs/protocol/agent-images.md` says. Measure it before the choice,
    because it sets the smallest real size.
  - On 22 November 2026 the user wrote: "user will be able to specify
    smaller options, lets not worry about the sizing issue for now". The
    user also wrote: "the user could choose a label later where a job only
    needs 1GB for example".
- **Problem:** A small label takes a whole slot of 2 threads and 8 GB. So
  small labels do not let a small machine run jobs, or run more of them.
- **Impact if no action:** A machine with 8 GB shows no slots for every
  label, and a Raspberry Pi 5 with 16 GB runs one job at a time.
- **Root question:** How does a machine count its capacity, so that each job
  holds only the threads and the memory of its size?
- **Options:**
  - **A.** Keep the slot rule. Small machines run no small jobs. Effort:
    Small.
  - **B.** Halve the slot to 1 thread and 4 GB on every platform. A machine
    with 8 GB then runs one job of up to 1 vCPU and 4 GB. A Raspberry Pi 5
    with 16 GB runs up to three such jobs. Existing labels keep their job
    counts, but every slot number in the dashboard and the docs doubles.
    Effort: Medium.
  - **C.** Fit each job by its own threads and memory, in place of slots. A
    job of 1 GB then fits wherever 1 GB and its threads are free. It
    changes the scaler, the slot allocation of the agent, the slot fields of
    the wire protocol, the store columns, the label dialog, and the docs of
    every platform. Effort: Large.
  - **D.** Halve the slot on Linux arm64 machines only, to 1 thread and
    4 GB. A label fits only machines of its own platform, so one fit never
    mixes two slot sizes. x64 machines and Macs keep their slot numbers.
    The slot functions in `crates/ciproxy-protocol` then take the platform,
    and the Machines page shows slots of two sizes. Effort: Medium.
- **Left out:**
  - Memory overcommit, because the host can then stop a job that uses all
    of its memory.
  - A smaller host reserve alone, because with any useful reserve an 8 GB
    machine keeps less than one slot of 8 GB.
- **Wider fix:** C, because it removes the fixed ratio of threads to memory
  on every machine.
- **Previous examples:** "Slots on a Mac" in `docs/protocol/job-routing.md`,
  and the mini PC example in `docs/protocol/job-routing-examples.md`.
- **Recommendation:** C. A label of 1 GB then fits wherever 1 GB is free,
  as the user expects. B and D still give such a label 4 GB.
- **Action:** ask the user, because the user chose to leave the sizing for
  later, and each option changes the slot rule.
