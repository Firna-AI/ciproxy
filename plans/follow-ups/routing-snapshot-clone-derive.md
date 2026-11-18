# Routing snapshots can still be copied

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from `conductor/final-review-r2-runtime`, the
  remaining part of option E of review finding 902.
- **Location:** `crates/ciproxy-store-interface/src/own_runners.rs:102`,
  the `#[derive(Clone)]` line of `RoutingSnapshot`.
- **Reason:** The user forbade edits under `crates/ciproxy-store-interface/`
  in the change that removed the last copy, so the derive stays until then.

- **Terms:**
  - **Routing snapshot:** a value of the `RoutingSnapshot` struct in
    `crates/ciproxy-store-interface/src/own_runners.rs`. It holds one
    consistent read of every routing input of one workspace.
  - **Preparation task:** the background task that
    `MachineRuntime::prepare_job` in
    `crates/ciproxy-control-plane/src/preparation.rs` starts for a job on an
    own machine. It sends the Docker budget and the job facts to the agent.
- **Background:**
  - `crates/ciproxy-store-pg/src/routing_snapshot.rs` loads a routing
    snapshot. Its `jobs` field holds every GitHub job row of the workspace.
  - The store keeps each finished job for 90 days, as "Data retention" in
    `docs/protocol/security.md` says. A snapshot also holds the last full
    report of each machine.
  - At commit `9b094e1`, `prepare_job` copied the whole snapshot into each
    preparation task, with `input.clone()` at `preparation.rs:34`. The task
    used the copy only to compute the Docker budget of one machine.
  - On 2026-11-17, the user chose to remove that copy, and to remove the
    `Clone` derive of `RoutingSnapshot`. Without the derive, each later copy
    is a compile error.
  - The change that added this item computes the Docker budget before it
    starts the task. It moves only narrow owned inputs into the task. After
    it, no code copies a routing snapshot.
  - The user forbade edits under `crates/ciproxy-store-interface/` in that
    change. So the `Clone` derive stays on `RoutingSnapshot`.
  - At `9b094e1`, a search found one copy of a routing snapshot, at
    `preparation.rs:34`. The call `snapshot.clone()` in
    `crates/ciproxy-control-plane/src/overflow_composition.rs` copies the
    Boat shared snapshot, which is another type.
- **Task:**
  - Remove `Clone` from the derive of `RoutingSnapshot`. Keep the derives of
    the other types in that file.
  - Build the workspace. Fix each compile error with a reference or with
    narrow owned inputs.
  - Do not add a manual `Clone` implementation or another full copy.
  - A test that needs a second snapshot calls its fixture builder again.
  - The change keeps the behaviour. It needs no protocol doc change and no
    new test, because the compiler is the guard.
  - Run `cargo build --workspace`. Then run `cargo xtask check` with
    `DATABASE_URL` set, as "Commands" in `AGENTS.md` says.
- **Impact if no action:** A later change can copy a whole routing snapshot
  into a background task again, and the compiler accepts it. Each copy keeps
  the job history of the workspace in memory while the task runs. The
  production control plane runs as one pod with a 1 GiB memory limit.
- **Needs:** A workspace that the user allows to edit
  `crates/ciproxy-store-interface/`.
- **Done when:** `RoutingSnapshot` has no `Clone` implementation, the
  workspace builds, and `cargo xtask check` passes.
