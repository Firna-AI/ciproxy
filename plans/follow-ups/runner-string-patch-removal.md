# Runner cleanup needs protected plan link cleanup

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from `conductor/ciproxy-mvp-milestone-13-e2e`,
  item 9 of the first owner review of pull request #31.
- **Updated:** 2026-11-07. Action A's code and regression are applied.
  Final link cleanup remains open.
- **Source:** Review.
- **Category:** Docs or spec.
- **Location:** `plans/ciproxy-mvp.md:884`, this follow-up item, and
  `plans/follow-ups/unused-retired-label-query.md:55`.
- **Reason:** The user reserved `plans/ciproxy-mvp.md` for the concurrent
  work of pull requests #36, #39, and #40. Its link prevents this item's
  deletion until that work settles.

- **Terms:**
  - **Source hook:** The C# file `image/runner-patch/CiproxyAssignment.cs`,
    which the image build compiles into the runner Worker before action setup.
  - **Worker:** `Runner.Worker.dll`, the GitHub Actions runner process that
    runs one job.
  - **Gateway helper:** `image/runner-patch/container-proxy-url.sh`, which
    prints the proxy URL on the job VM's Docker bridge gateway.
- **Background:**
  - The user approved option A on 2026-11-06. It removes the old runner
    scripts and old-only test, with applicable coverage on the source-hook
    path. The scripts are entry points, so that approval covers their removal.
  - Production already used the source hook. The old scripts stayed beside
    it, and image provisioning copied them into `/opt/ciproxy-image/`.
  - On 2026-11-07, the cleanup applied A's code, docs, and socket regression.
    The sections below record that work and the original decision.
  - The completed milestone text in `plans/ciproxy-mvp.md` links this item
    through `follow-ups/runner-string-patch-removal.md`.
  - The ownership audit found that plan in all three protected open PR file
    lists. The user requires those files to stay unchanged. The plan keeps
    its original words and link target. This item therefore stays open.
- **Problem:** The code cleanup is applied, but its final record cannot yet
  close. Deleting this item would break the protected plan's history link.
- **Impact if no action:** The functional fix remains available. The
  follow-up remains open until the protected link can change. Deleting it
  early would leave the plan with a broken link.
- **Root question:** How should the approved runner cleanup reach closure?
- **Options:**
  - **A.** Keep the applied cleanup and its tests. After the protected work
    settles, preserve the plan's history link with a permalink. Update the
    remaining references and delete this item. Effort: Small.
- **Left out:** Restoring the old scripts reverses approved A. Deleting
  this item now breaks a protected link. Editing the protected plan now
  violates the user's file reservation. Only final closure remains, so
  there is no new implementation choice.
- **Wider fix:** None additional. The existing replacement and history-link
  rules already require complete removal and valid historical references.
- **Previous examples:** Commit `ec5344c` added the old scripts on
  2026-11-04. Commit `703b9b1` added the source hook and compatibility text
  on 2026-11-06. This item records that retained-old-path case.
- **Related:** The
  [cache and Docker live proof](agent-cache-area-integration.md) item owns
  the authenticated `cache` scenario on a KVM host. Its work and this link
  closure do not wait for each other.
- **Recommendation:** A. It preserves the approved cleanup and respects the
  protected files until their concurrent work settles.
- **Action:** fix after merge, apply A, because final closure needs the
  protected plan file after its concurrent work settles.

## Applied Action A

The 2026-11-07 cleanup removes exactly these four files:

- `image/runner-patch/patch-runner.py`
- `image/runner-patch/run-with-cache.sh`
- `image/runner-patch/job-started.sh`
- `crates/ciproxy-cache-sidecar/tests/runner_environment_tests.rs`

The first script renamed three stock runner environment names at equal
length in the managed string table. The shell wrapper exported proxy URLs
and a chained job-started hook. That hook ran a prior hook and wrote proxy
URLs to `GITHUB_ENV`. No image script, systemd unit, or smoke called them.
The old wrapper test was their only repository consumer.

The full repository searches excluded this item. They found no reference
to `patch-runner.py`. Only the wrapper named `job-started.sh`, and only its
test named `run-with-cache.sh`. The operator scripts still counted as
functionality. The user's 2026-11-06 approval authorized their removal.
The post-cleanup searches found no remaining consumer outside this item.

The cleanup preserves these live parts:

- The gateway helper. `image/provision.sh` installs it as
  `/usr/local/libexec/ciproxy-container-proxy-url`.
  `crates/ciproxy-guest/src/guest_setup.rs` runs it before job receipt.
- `CiproxyAssignment.cs`, `prepare-source.py`, `LICENSE`, and the `smoke/`
  and `tests/` directories under `image/runner-patch/`.
- The sidecar's `tempfile` dependency, which its `src/` still uses.
  Cargo discovers the removed test without a manifest entry.
- The directory name and component name "runner patch" in the root README,
  `docs/architecture.md`, and the cache-sidecar README.
- The completed milestone wording in `plans/ciproxy-mvp.md` and every
  protocol doc. The product behavior does not change.

The runner patch README now describes a Worker built from pinned source.
It removes the managed-string-table and PE-offset claim, compatibility
smoke text, and `JobHookProvider` link. Only the removed chained hook used
that linked `GITHUB_ENV` contract. The README and `smoke/live-check.py` now
read `ACTIONS_*` values only. The old `ZCTIONS_*` fallback is removed.
The authenticated smoke now explicitly requires a stock runner. Steps of
an active source hook see the proxy URL, rather than the original URLs.

The live Related text in `agent-cache-area-integration.md` now links the
retained runner cache guide and records that the old scripts are removed.
The plain history reference in `unused-retired-label-query.md` remains
valid while this item stays. The protected MVP plan remains unchanged.

## Preserved source-hook contract

`image/build-runner.sh` builds the pinned Worker after `prepare-source.py`
adds the source hook. `image/provision.sh` installs stock and own-machine
Workers side by side. `image/runner-entry.sh` selects the own-machine
Worker when `CIPROXY_ASSIGNMENT_SOCKET` is set and runs its format probe.
Otherwise it selects the stock Worker. Boat uses that stock Worker.

The source hook reads the original runtime token, cache URL, results URL,
and cache rights before action setup. It sends them to `ciproxy-guest`
over `/run/ciproxy/assignment.sock`, in handoff format 2. The guest measures
the proxy URL on the actual Docker bridge before the job arrives.
The hook waits for the host decision with no time limit of its own.

On `cache_ready`, the hook sets both `CacheServerUrl` and
`ResultsServiceUrl` to the one container-reachable proxy URL. On
`github_only`, it keeps the original URLs. It never changes the token.
The opt-in `image/runner-patch/tests/host-decision.py` runs the actual hook
against host-decision fixtures. It remains outside the gate.

## Regression and packaging boundaries

`crates/ciproxy-guest/tests/assignment_socket.rs` is the integration target.
It imports private production modules without a public API or a new
dependency. Its case lives in `tests/_tests_/assignment_socket_tests.rs`
beside that target. It drives the actual `SocketAssignments`, `SocketReply`,
and `handoff::read` over a temporary Unix socket. The guest README names
that boundary.

The fixture uses the exact source-hook JSON layout. It checks the original
token, original URLs, cache permissions, and fixed preloaded Docker facts.
Each preparation and decision line carries format 2 and the job UUID.
It checks `preparing`, acceptance of a matching `acknowledged` line, and
the final `decision`. `cache_ready` carries one proxy URL.
`github_only` carries none. Neither final response contains a token.
Run the case from the repository root:

```sh
cargo test --package ciproxy-guest --test assignment_socket \
  source_hook_socket_preserves_credentials_and_returns_host_decisions \
  -- --nocapture
```

The removed wrapper assertions covered prior-hook chaining, exit status,
`GITHUB_ENV`, forced cache V2, a forced trailing slash, and shell environment
import. Those behaviors left with the wrapper. This cleanup adds no .NET CI
toolchain or gate phase. The existing .NET check remains opt-in.

The offline packaging proof uses production provisioning's `cp -a` and
production export, archive, ext4, and package commands. It checks the
release tree, Boat archive, and Firecracker ext4 filesystem. Other
provisioners and libguestfs transport remain fixture boundaries.

The retained cache-and-Docker item owns the authenticated KVM proof.
Its second cache job must restore the first job's entry in a fresh VM from
the machine cache. This closure does not replace or close that live work.

## Original decision and approval

The original review graded the code cleanup Low severity and Medium
effort, with High confidence and no harder-later cost. On 2026-11-06, the
user chose A for after the merge. Production already used the new path,
and the cleanup reached unchanged operator scripts.
The original options were:

- **A.** Remove the old scripts, old-only test, and compatibility references.
  Preserve applicable coverage through the source-hook path. Effort: Medium.
- **B.** A, and move the token and URL assertions into a .NET test of
  `CiproxyAssignment.cs` against pinned runner source in a new gate phase.
  It adds the .NET SDK, a runner-source download, and a CI gate phase.
  That needs separate approval. Effort: Large.
- **C.** A without new gate coverage. Keep the live `cache` scenario as the
  only source-hook proof. The Worker-to-guest socket contract then lacks a
  gate test. Effort: Small.

The original recommendation was A: maintain one integration and test the
real socket in Rust. Keeping the scripts, renaming their interface, or
excluding them only from the image would retain the replaced path.
No smoke consumed them. The existing replacement rules were the wider fix.
Option B remains unapproved. This closure adds no new decision.

## Remaining closure

1. Wait until the concurrent work of #36, #39, and #40 settles. Recheck
   their current file ownership and the user's reservations before editing
   `plans/ciproxy-mvp.md`. Its presence in all three protected file lists
   blocked even the link-only change on 2026-11-07.
2. If the plan still links this item, replace only the link target with a
   GitHub permalink to this item at a commit where it exists. Base commit
   `54816e311b24d241010efcb1f6141bc2524dad1e` preserves the original finding
   and approved A. Keep the historical wording unchanged.
3. Replace the plain history reference in `unused-retired-label-query.md`
   with that same retained history link. Search all docs, plans, and READMEs
   for other references before deletion. Keep any live source-hook links.
4. Confirm the applied code and regression remain. Record the four removals
   and the user's approval in the cleanup commit or PR. Keep the current
   implementation's complete-gate result with its orchestrator.
5. Delete this item only after its remaining references are safe. Run the
   Markdown/link checks and deletion audit on that closure change. A docs-only
   closure does not require another Cargo test run.

- **Done when:** The applied A code and socket regression remain verified.
  The implementation's complete gate passes. The protected file work has
  settled, and no protected edit is required to preserve this item's links.
  The remaining references point to retained history or live docs. The
  closure change deletes this item and passes Markdown/link and deletion
  audits. The separate authenticated KVM proof remains in its own item.
