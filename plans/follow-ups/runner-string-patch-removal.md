# The old runner patch remains

- **Severity:** Low. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/ciproxy-mvp-milestone-13-e2e`,
  item 9 of the first owner review of pull request #31.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `image/runner-patch/patch-runner.py`,
  `image/runner-patch/run-with-cache.sh`,
  `image/runner-patch/job-started.sh`,
  `crates/ciproxy-cache-sidecar/tests/runner_environment_tests.rs`,
  `image/runner-patch/README.md` lines 4-5, 49-51, 109-111, and 147-148,
  `image/runner-patch/smoke/live-check.py:17-21`, and
  `crates/ciproxy-guest/src/assignment.rs:97-132`.
- **Reason:** On 2026-11-06, the user chose this work for after the merge,
  because production uses the new source hook, and cleanup reaches unchanged
  operator scripts.

- **Terms:**
  - **Source hook:** The C# file `image/runner-patch/CiproxyAssignment.cs`,
    which the image build compiles into the runner Worker to run before
    action setup.
  - **String patch:** The script `image/runner-patch/patch-runner.py`, which
    renames three environment names inside the stock `Runner.Worker.dll` at
    equal length.
  - **Shell wrapper:** The script `image/runner-patch/run-with-cache.sh`,
    which exports the proxy URLs and the chained hook, then starts the
    runner.
  - **Chained hook:** The runner job-started hook
    `image/runner-patch/job-started.sh`, which runs an earlier hook, then
    writes the proxy URLs to `GITHUB_ENV`.
  - **Gateway helper:** The script `image/runner-patch/container-proxy-url.sh`,
    which prints the proxy URL on the Docker bridge gateway of the job VM.
- **Background:**
  - A job VM on a customer's own machine sends the cache calls of its job
    steps to a proxy in the VM.
  - The runner Worker, `Runner.Worker.dll`, is the GitHub Actions runner
    process that runs one job.
  - `image/build-runner.sh` builds the Worker from the pinned runner source,
    after `prepare-source.py` adds the call of the source hook.
  - `image/provision.sh:13-14` installs the stock Worker and the
    own-machine Worker side by side.
  - `image/runner-entry.sh:13-17` selects the own-machine Worker when
    `CIPROXY_ASSIGNMENT_SOCKET` is set, and the stock Worker otherwise.
    Boat, the rented overflow provider, runs the stock Worker.
  - The source hook sends the original runtime token, cache URL, results
    URL, and cache permissions to the guest program, `ciproxy-guest`, over
    `/run/ciproxy/assignment.sock`.
  - When the guest replies with a proxy URL, the hook sets both
    `CacheServerUrl` and `ResultsServiceUrl` to it, at
    `CiproxyAssignment.cs:59-63`. It never changes the token.
  - The string patch, the shell wrapper, and the chained hook are the
    earlier way. No image script, systemd unit, or smoke runs them.
  - `image/provision.sh:25` copies the whole image definition to
    `/opt/ciproxy-image/`, so the old scripts still ship in the image.
  - The wrapper test,
    `crates/ciproxy-cache-sidecar/tests/runner_environment_tests.rs`, runs
    the shell wrapper with a fake runner in the `rust-test` gate phase.
  - The live `cache` scenario of `crates/ciproxy-agent/scripts/own_cache.py`
    proves the source hook end to end. Its second job, in a fresh VM, must
    restore the entry of the first job from the machine's own cache.
  - On 2026-11-06, the user chose option A for after the merge.
- **Problem:** The old scripts remain beside the replacement.
  `image/runner-patch/README.md:49-51` keeps them for a compatibility smoke,
  but no smoke calls them. The gate still runs the wrapper test, which tests
  only the shell wrapper.
- **Impact if no action:** The repository maintains two runner
  integrations. The gate keeps a test of the replaced path, and readers can
  take the old scripts for the live mechanism.
- **Root question:** Which runner integration should the repository
  maintain?
- **Options:**
  - **A.** Remove the old scripts, old-only test, and compatibility
    references. Preserve applicable coverage through the source-hook path.
    Effort: Medium.
  - **B.** A, and move the token and URL assertions into a .NET test of
    `CiproxyAssignment.cs` against the pinned runner source, in a new gate
    phase. It adds the .NET SDK, a runner-source download, and a gate phase
    to CI. That needs a separate user approval. Effort: Large.
  - **C.** A without new gate coverage. Keep the live `cache` scenario as
    the only proof of the source hook. The worker-to-guest socket contract
    then has no gate test. Effort: Small.
- **Left out:** Keeping the old scripts fails the goal, because only removal
  completes the replacement. Wrapping the new hook in the old interface keeps
  the old layer. The compatibility smoke is no consumer.
  `image/runner-patch/smoke/live-check.py:17-21` reads `ZCTIONS_*` first,
  but it falls back to the `ACTIONS_*` values of a stock runner. Scripts that
  stay in the repository but leave the image still form a second way.
- **Wider fix:** None additional. The `plans` and `rust` skills already
  require the complete replacement of an old mechanism.
- **Previous examples:** No earlier leftover case found. Commit `ec5344c`
  added the old scripts on 2026-11-04. Commit `703b9b1` added the source
  hook and README lines 49-51 on 2026-11-06.
- **Related:** The
  [cache and Docker live proof](agent-cache-area-integration.md) item runs
  the live `cache` scenario on a KVM host. This item keeps that scenario as
  the end-to-end proof of the source hook. Neither item waits for the other.
- **Recommendation:** A. One maintained integration best protects the
  codebase, and one Rust test of the real socket keeps the coverage that the
  gate can run.
- **Action:** fix after merge, apply A, because production uses the new
  path and cleanup reaches unchanged operator scripts.
- **Steps for A:**
  1. Repeat the searches. `git grep` finds no file that names
     `patch-runner.py`. It finds `job-started.sh` only in
     `run-with-cache.sh`, and `run-with-cache.sh` only in the wrapper test.
  2. Delete exactly these four files:
     - `image/runner-patch/patch-runner.py`
     - `image/runner-patch/run-with-cache.sh`
     - `image/runner-patch/job-started.sh`
     - `crates/ciproxy-cache-sidecar/tests/runner_environment_tests.rs`.
       Cargo finds test files without a manifest entry, so
       `crates/ciproxy-cache-sidecar/Cargo.toml` needs no change. Keep its
       `tempfile` dependency, because the sidecar's `src/` uses it.
  3. Keep these files, because the source hook uses them:
     - The gateway helper. `image/provision.sh:20` installs it as
       `/usr/local/libexec/ciproxy-container-proxy-url`, and
       `crates/ciproxy-guest/src/handoff.rs:67` runs it to get the proxy
       URL.
     - `CiproxyAssignment.cs`, `prepare-source.py`, `LICENSE`, and the
       `smoke/` directory in `image/runner-patch/`.
  4. Remove the text of the old way from `image/runner-patch/`:
     - `README.md:49-51` says that the string patch and the shell wrapper
       stay for a compatibility smoke. Remove it.
     - `README.md:4-5` says "It keeps the runner's managed string table and
       PE offsets in place". Say instead that the image installs a Worker
       built from source.
     - `README.md:109-111` and `smoke/live-check.py:17-21` read `ZCTIONS_*`
       names, which only the patched runner wrote. Read `ACTIONS_*` only.
     - In that README text, say that the live smoke runs on a stock runner.
       With the source hook active, steps see the proxy URL, not the
       original URLs.
     - `README.md:147-148` links the runner `JobHookProvider` contract. Only
       the `GITHUB_ENV` lines of the chained hook used it. Remove it.
  5. Leave these unchanged:
     - The component name "runner patch" in `README.md:39`,
       `README.md:167`, `docs/architecture.md:245`, and
       `crates/ciproxy-cache-sidecar/README.md:217`. It names the Apache-2.0
       directory `image/runner-patch`, which keeps the source hook. Do not
       rename the directory.
     - The completed milestone text about the patch in
       `plans/ciproxy-mvp.md`, while that plan exists. It is history.
     - The files under `docs/protocol/`. None names the patch, the wrapper,
       or the hook, and the product behaviour does not change.
  6. Move the coverage that still applies to the source-hook path:
     - Port two assertions of the wrapper test. Job steps keep the original
       `ACTIONS_RUNTIME_TOKEN`. The cache URL and the results URL become one
       container-reachable proxy address.
     - Do not port the wrapper-only assertions: hook chaining, its exit
       status, the `GITHUB_ENV` lines, the forced `ACTIONS_CACHE_SERVICE_V2`,
       and the added trailing slash. That behaviour goes with the wrapper.
     - Add one Rust test in `ciproxy-guest`. Drive the real
       `SocketAssignments` and `SocketReply` over a temporary Unix socket
       with the exact lines of the source hook.
     - Assert the `{"preparing":true}` line, the acceptance of
       `{"acknowledged":true}`, and a last line with only `proxy_url`, or
       `{}` on fallback. A reply without a token gives the hook no token to
       change.
     - `handoff::read` also runs the gateway helper and `du`. Both fall back
       on failure, so assert only the socket lines.
     - Take the lines from `CiproxyAssignment.cs` and
       `crates/ciproxy-guest/src/assignment.rs` when you start. The handoff
       can change before then.
     - Break one literal once and see the test fail on an assertion. Run the
       test 10 times, because it is asynchronous.
     - Keep the live `cache` scenario as the end-to-end proof. Only the
       image build compiles the source hook.
     - Do not add a .NET project, a CI toolchain, or a gate phase. That is
       option B, and it needs a separate approval.
  7. Record the deletion in the commit or the pull request description. The
     scripts are entry points, so they are functionality. The user's choice
     of A on 2026-11-06 is the approval. Give the searches of the first
     step. Run the deletion audit of the `git-commit` skill.
  8. Check the links. No Markdown link points to the removed text or the
     scripts. Only `crates/ciproxy-cache-sidecar/README.md:217` links the
     runner patch README, and that README stays. Wrap lines at 80
     characters.
- **Done when:** The four files are gone. The runner patch README and
  `live-check.py` name only `ACTIONS_*` values. The new guest socket test
  failed once on a broken literal and passed 10 of 10 runs. The complete
  gate passes.
