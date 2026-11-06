# Synchronous store calls can block Tokio workers

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/runner-observation-confirmation`,
  the wider fix of finding 3 of the second owner review of pull request #31.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-control-plane/src/composition.rs:51`, the
  store calls of `crates/ciproxy-control-plane/src/`,
  `crates/ciproxy-store-pg/src/connection.rs:11-23`, and
  `crates/ciproxy-store-pg/README.md:142-143`.
- **Reason:** The handle reaches about 20 store call sites in 11 files,
  including `runtime.rs`, `lifecycle.rs`, and `composition.rs`, which the
  organization-add branch of pull request #32 changes. The narrow listener
  fix needs none of it.

- **Terms:**
  - **Store:** The `Store` trait of `crates/ciproxy-store-interface`. Its
    methods are synchronous.
  - **Blocking worker:** A thread of the Tokio blocking pool, which
    `tokio::task::spawn_blocking` uses.
  - **Async store handle:** A private type that holds the store and runs
    each store closure on a blocking worker.
- **Background:**
  - `crates/ciproxy-store-pg/README.md:142-143` says: "Queries are
    synchronous. Async services must call them from blocking workers."
  - The PostgreSQL store holds one `std::sync::Mutex<PgConnection>` for the
    whole process (`crates/ciproxy-store-pg/src/connection.rs:11-23`). Each
    query takes that lock and runs a synchronous Diesel query.
  - The control plane keeps the store as a direct `Arc<dyn Store>`, for
    example the `store` field of `Hosted` in `composition.rs:51`. Each async
    caller wraps its call in `tokio::task::spawn_blocking`, by convention
    only. The type also permits a direct call on a Tokio worker.
  - `crates/ciproxy-github/src/persistence/mod.rs:30-38` already has a
    `query` helper that runs a store closure with `spawn_blocking`.
  - The original problem: at `5bfb46f`, each listener start in
    `Hosted::supervise` read the organization directly on a Tokio worker
    (`crates/ciproxy-control-plane/src/listeners.rs:134`). It only kept the
    installation for the missing-scale-set repair. The call took the shared
    connection lock and ran a blocking query, and a store error there ended
    serving.
  - The direct fix: on 2026-11-06, the user approved returning the
    installation ID with each discovered listener target. The change that
    added this item applies that fix and removes the read. No other direct
    call was known in the control plane.
- **Problem:** Nothing in the control plane stops a new synchronous store
  call on a Tokio worker. The listener read showed that the convention can
  fail, and only a code review found it.
- **Impact if no action:** A later direct call can hold a runtime thread
  while it waits for the shared connection. Each review must check each new
  store call by hand.
- **Root question:** How does the control plane make a synchronous store
  call on a Tokio worker impossible, not only a convention?
- **Options:** Only F and G prevent the class. The other approaches fix one
  call or only reduce the cost, as **Left out** says.
  - **F.** Keep the control-plane store only behind a private async store
    handle, like the `ciproxy-github` helper. A direct store call on a Tokio
    worker then does not compile in the crate. Public constructors keep
    `Arc<dyn Store>` and wrap it, so the test support stays. It reaches
    about 20 call sites in 11 files. Effort: Medium.
  - **G.** Replace the store client with an async client and pool, for
    example `diesel-async`, and make the store traits async. This removes the
    class in every crate and the one shared connection. It changes every
    store crate and consumer, and the store design of
    `crates/ciproxy-store-pg/README.md`. Effort: Large.
- **Left out:**
  - The local options for the listener read: a `spawn_blocking` around it,
    a read only after the error, an identity from the GitHub listener, and
    an App refresh by workspace. They fix one call, and the direct fix
    already removes it.
  - A lint, a gate, or a test for the class. It adds a check that needs the
    user's approval, and F prevents the class by its type.
  - `tokio::task::block_in_place` around each direct call. It still blocks
    a runtime thread, and it fails on a current-thread runtime.
  - More Tokio worker threads. Each direct call still blocks a thread.
  - No change. The class stays possible.
- **Wider fix:** F for the control plane, and G for every crate.
- **Previous examples:** The listener start read in `Hosted::supervise`,
  which commit `703b9b1` of pull request #31 added. It was the only such
  call found in the control plane.
- **Related:**
  [The serving shutdown item](control-plane-serving-shutdown-regression.md)
  covers the stop path of `Hosted::supervise`, the same code. Until it
  lands, no test protects that stop path, so F must keep it unchanged.
  Neither item waits for the other.
- **Recommendation:** F. It makes the wrong call fail to compile in the
  control plane, with no change to public constructors or other crates. G
  best protects every crate, but it changes the store design everywhere.
- **Action:** fix after merge, apply F, because it reaches files that the
  organization-add branch changes, and the direct listener fix needs none of
  it. F keeps the behaviour, so it adds no test. Run the existing suites.
