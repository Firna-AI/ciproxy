# Label size conversions repeat in nine places

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Fix after merge.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 1008, with the
  byte-to-GB cut-off in `start_request` that its check found.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-store-interface/src/vm_size.rs`,
  `VmSize::bytes` and `VmSize::from_request`,
  `crates/ciproxy-api/src/handlers/label_views.rs`, `fit`,
  `crates/ciproxy-api/src/handlers/label_boat.rs`, `enrich`,
  `crates/ciproxy-api/src/handlers/machines.rs`, `read`,
  `crates/ciproxy-control-plane/src/snapshot.rs`, `size` and `build`,
  `crates/ciproxy-control-plane/src/docker_budget.rs`, `policy`,
  `crates/ciproxy-control-plane/src/boat_snapshot.rs`, `project`,
  `crates/ciproxy-overflow/src/service/start.rs`, `start_request`, and
  `crates/ciproxy-overflow/src/service/provider_preparation.rs`,
  `selected_size`.
- **Reason:** The severity is Low, and the fix reaches nine call sites in
  three crates.

- **Terms:**
  - **Whole-GB request:** the size that a label saves: the CPU count, the
    memory in decimal GB, and the free job disk in decimal GB. `VmSize` in
    `vm_size.rs` holds it as three `i32` counts.
  - **Byte size:** `ciproxy_protocol::resources::VmSize`, the size in bytes
    that the scaler, the agent, and the wire use.
  - **`GIGABYTE`:** the constant of `ciproxy_protocol::resources`,
    1,000,000,000 bytes.
  - **Delivery record:** `StartRecord`, the saved record of one Boat start.
    It keeps the requested size as three `u32` counts:
    `requested_vcpus`, `requested_memory_gb`, and `requested_disk_gb`.
  - **"When nothing fits":** the label setting that chooses between a wait
    and the largest Boat size. The wire, the store, and the scaler each
    have their own enum for it.
- **Background:**
  - `VmSize::bytes` in `crates/ciproxy-store-interface/src/vm_size.rs`
    turns a stored whole-GB request into a byte size. Commit `703b9b19` of
    6 November 2026 added it. No code calls it.
  - For a negative count, `bytes()` returns an error. Each copy below gives
    zero for a negative count.
  - `VmSize::from_request` turns a byte size back into whole GB. It refuses
    a byte size that is not a whole number of GB.
  - Three API copies turn whole GB into bytes: `fit` in `label_views.rs`,
    `enrich` in `label_boat.rs`, and the size of the default label in
    `read` of `crates/ciproxy-api/src/handlers/machines.rs`. They also turn
    the unsaved preview input of the label dialog into bytes.
  - `size` in `crates/ciproxy-control-plane/src/snapshot.rs` is a fourth
    copy. Three call sites use it. Two are in `build` of the same file, for
    the label size and the requested size of a runner. One is in `policy`
    of `docker_budget.rs`.
  - Three Boat copies turn the counts of a delivery record into bytes:
    - the retry check at the top of `start_request` in
      `crates/ciproxy-overflow/src/service/start.rs`
    - `selected_size` in `provider_preparation.rs`, in the same folder
    - `project` in `crates/ciproxy-control-plane/src/boat_snapshot.rs`
  - `BoatDeliveryGuard` copies the same three counts. `bytes()` takes only
    the `i32` store type, so it cannot take these `u32` counts without a
    new fallible step at each copy.
  - Three callers return no `Result`: `label_views::fit`,
    `label_boat::enrich`, and `snapshot::build`. A move to `bytes()` needs a
    new error branch at each of them. A branch that refuses the value is
    stricter validation, and stricter validation needs the user.
  - One copy goes the other way. When `start_request` saves a new delivery
    record, it divides the requested bytes by 1,000,000,000. The integer
    division cuts off a part that is not a whole GB.
  - A retry of the same delivery compares the saved counts, times
    1,000,000,000, with the requested bytes. A cut-off value never matches,
    so the retry fails with `Error::DeliveryConflict`.
  - The scaler builds each request from a whole-GB label, through
    `snapshot::size`. So no current start meets the cut-off.
  - `label_views::stored` refuses a label count below one. So no product
    path saves a negative or zero count.
  - The API maps the wire "When nothing fits" to the store enum. `build`
    and `selected_size` each map the store enum to the scaler enum. Each
    map is an exhaustive `match`, so a new variant stops the compiler at
    each copy.
  - The scaler keeps storage out, as its README says. The store interface
    depends only on the protocol crate. The Apache-2.0 protocol crate holds
    no hosted policy. So one shared map of "When nothing fits" needs a new
    crate dependency or an unrelated home.
  - Two SQL statements compare a stored label row with a stored JSON size:
    `create_boat_runner_intent` in
    `crates/ciproxy-store-pg/src/boat_runtime.rs`, and the `ACCOUNT` query
    of `crates/ciproxy-store-pg/src/overflow/authority.rs`.
  - `label_boat::enrich` is the one Boat fit for Settings and Jobs. Every
    option keeps it as that one fit, and changes only how it gets its byte
    size.
- **Problem:** The dashboard fit, the scaler snapshot, and the Boat
  delivery turn whole GB into bytes by hand, at nine call sites. The copies
  give zero for a bad count, where `bytes()` gives an error. The reverse
  copy in `start_request` cuts off a byte size that `from_request` refuses.
- **Impact if no action:** Each new consumer can add one more copy, with
  its own result for a bad value. A later request that is not a whole GB
  would start a VM of the cut-off size, and its retry would fail.
- **Root question:** How does every consumer convert label sizes between
  whole GB and bytes by one rule, with one result for a bad value?
- **Options:**
  - **A.** Replace the API copies with `bytes()`. Keep the current handling
    of unavailable facts. Cost: a new error branch at each caller that
    returns no `Result`. Effort: Small.
  - **B.** Move every affected consumer to `bytes()` and to one shared map
    of "When nothing fits". Remove their local conversions. Cost: `bytes()`
    cannot take the `u32` counts of the delivery record. B names no home
    for the shared map, and each home needs a new crate dependency or an
    unrelated crate, such as `ciproxy-overflow`. Effort: Medium.
  - **C.** Use one canonical request type through the label storage, the
    runtime, and the API composition. Cost: it changes many more
    interfaces. It must drop the documented whole-GB request type, or move
    that type to a crate that the scaler can use. Effort: Large.
  - **D.** One total conversion that every current copy uses.
    - Give `vm_size.rs` one conversion from unsigned whole-GB counts to
      bytes, built on `GIGABYTE`. The conversion cannot fail.
    - Make `bytes()` call it. For a negative count, keep the result of
      today's copies, zero.
    - Use it in the three API copies, at the three call sites of
      `snapshot::size`, and in the three Boat delivery copies.
    - Give the reverse step, bytes to whole GB, one shared rule too. Use it
      where `start_request` saves a new delivery record, and where it
      compares a retry with that record. Then the saved record and the
      retry check cannot disagree.
    - Keep today's cut-off in that reverse rule. A refusal is a separate
      question, under **Left out**.
    - Keep the three exhaustive matches of "When nothing fits".
    - D drops the assumption that `bytes()` is a ready owner whose use
      keeps today's results. The first review of this problem assumed it.
    - Cost: no change to stored data, to crate dependencies, or to the
      result of any current request.
    - Effort: Medium.
- **Left out:**
  - Another conversion helper for the API only: it fails the goal, because
    the runtime and Boat copies stay.
  - `VmSize` fields that refuse a count below one: for this goal, D is
    better on every point. D gives the same one conversion with less change
    and no user decision. Whether a bad stored size must fail is a separate
    question of stricter validation for the user.
  - A refusal at the Boat start of a requested byte size that is not a
    whole GB: it is stricter validation. So it is a separate question for
    the user. No current path sends such a size.
  - One "When nothing fits" enum in `ciproxy-protocol`: it breaks the
    repository rule that the Apache-2.0 crates hold no hosted code.
  - A Rust conversion inside the two SQL statements: not a real choice.
    Each statement compares a stored label row with a stored JSON field.
- **Wider fix:** D, changed from B.
  - B is a helper that the Boat delivery copies cannot use. D is a helper
    that every current call site uses.
  - C can remove the whole-GB values from runtime code. But it must drop
    the documented whole-GB request type, or move it to a crate that the
    scaler can use. So C does not rank higher.
  - The rule of the `delegating` skill for shared pieces of parallel work,
    added on 20 November 2026, does not cover this class. The copies did
    not come from parallel branches. They came
    one after another on one line of history, in `599d240b`, `a8697a1d`,
    `703b9b19`, `fa721d4c`, and `85b1c288`, from 5 to 8 November 2026.
- **Previous examples:** Commit `3b082ee` of 17 November 2026 removed a copy
  of the Boat fit from the job view. It made `label_boat::enrich` the one
  Boat fit for Settings and Jobs.
- **Related:** `plans/follow-ups/late-start-label-policy.md`: the same code,
  under its option D only. That option reads a fresh scaler snapshot
  through `snapshot::size`. D here replaces that function with the same
  result, so the options do not conflict.
- **Recommendation:** D, changed from B. D gives every current copy one
  total conversion with one result for a bad value. It also reaches the
  Boat delivery copies that `bytes()` cannot take, and it needs no new
  error branch and no new crate dependency.
- **Action:** fix after merge, apply D, because the severity is Low, and
  the fix reaches nine call sites in three crates.
