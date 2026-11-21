# Local checks bypass the Boat entry installation

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-runner-start`, review round
  2, finding 7. Its finding check corrected the impact and the root
  question, and added option D.
- **Source:** Review.
- **Category:** Test.
- **Location:** `image/provision.sh:19-26`,
  `image/check-boat-runner.py:86-94`,
  `image/check-runner-environment.py:71-72`, and
  `image/_checks_/boat_fixture.py:76-78`.
- **Reason:** The restore smoke catches a wrong installed entry before
  publication, so the gap costs only a late, paid detection.

- **Terms:**
  - **Guest:** the ciproxy program in the image that starts the runner of
    one job and reports the end of that runner.
  - **Boat entry:** `image/boat-runner-entry.sh`, the wrapper that sends
    the runner's stderr to a journal helper, `image/boat-runner-stderr.py`,
    and then runs the common entry.
  - **Common entry:** `image/runner-entry.sh`, which loads the shell
    settings of the `runner` account and starts the GitHub runner's
    listener.
  - **Native Boat build:** the paid build that installs the image on a Boat
    sandbox and saves it as a Boat snapshot. Boat is the overflow provider.
  - **Restore smoke:** the manual check in "Restore without a
    registration" of `image/boat-smoke.md`, on a sandbox restored from a
    new snapshot.
- **Background:**
  - The guest starts `/usr/sbin/runuser --user runner --` with its entry
    and a cleared environment, at
    `crates/ciproxy-guest/src/process.rs:63-83`. Its default entry is
    `/usr/local/libexec/ciproxy-runner`, at
    `crates/ciproxy-guest/src/cli.rs:26`. The Boat guest unit passes no
    other entry, at `image/ciproxy-boat-guest.service:10`.
  - In Boat mode, `image/provision.sh:19-26` installs the Boat entry at
    that path. It installs the common entry as
    `/usr/local/libexec/ciproxy-runner-common`. It also installs the journal
    helper and the error hook `image/boat-runner-errors.sh`. Machine mode
    installs the common entry at the guest's path. "The ciproxy parts" in
    `image/boat-build-steps.md` states this install.
  - `prepare_root` in `image/check-boat-runner.py` copies the four Boat
    runner files with its own table of names and modes, at lines 86-94.
    Each case then starts the Boat entry, the journal helper, or the
    common entry directly under `chroot`.
  - `image/check-runner-environment.py:71-72` copies the common entry to
    the guest's path. That check starts the real guest in Boat mode at
    lines 108-111.
  - With a registration, the launch fixture only copies the delivered
    file, at `image/_checks_/boat_fixture.py:76-78`. It starts the real
    guest only when no registration exists.
  - No local check runs the Boat branch of `provision.sh`.
    `image/_checks_/boat_locale.py:38` replaces the script with `exit 0`.
    `image/check-export.py:54` runs it in machine mode only, and its stub
    `install` copies only the guest unit, at
    `image/_checks_/export_fixture.py:47-51`.
  - So a change that installs the common entry at the guest's path in Boat
    mode passes every local check. A read of the code traced this. Nobody
    ran it.
  - The restore smoke runs the guest's command with a cleared
    environment, at `image/boat-smoke.md:122`. It requires the journal
    line `[ciproxy] runner entry error: JIT registration required`, at line
    135. With the common entry at the guest's path, that error goes to the
    stderr of the command, and no journal record appears. So the smoke
    fails before publication.
- **Problem:** No local check installs the Boat runner files through the
  real Boat branch of `image/provision.sh`. So a change that breaks that
  install passes every local check.
- **Impact if no action:** Such a change shows only when the restore smoke
  fails, after a full native Boat build and a restore sandbox. The smoke
  catches it before publication, so the harm is a late, paid detection.
- **Root question:** How can a change that breaks the Boat runner install
  fail before a paid build and restore?
- **Options:**
  - **A.** Run the real Boat branch of `image/provision.sh` in the private
    root of `image/check-boat-runner.py`, in place of its copy table. Keep
    the direct entry calls of the cases. A catches a wrong install path or
    mode before the build. It does not run the real guest. Effort: Medium.
    The steps:
    1. Run `provision.sh boat` as root under `chroot` in the private root,
       before the cases change to the runner user.
    2. Use the real `install`, `chmod`, `rm`, and `dirname`. Stub `id`,
       `useradd`, `usermod`, `chown`, `tar`, `sha256sum`, and `find`.
       `image/_checks_/export_fixture.py` gives the stub pattern.
    3. Copy the definition files that the Boat branch of `provision.sh`
       reads, with `pins.env`. Create the four `/tmp` inputs:
       `ciproxy-runner.tar.gz`, `ciproxy-worker.dll`, `ciproxy-guest`, and
       `ciproxy-cache-sidecar`. Create the `Runner.Worker.dll` that the
       stub archive does not unpack. Create `/etc/sudoers.d`,
       `/etc/systemd/system`, and `/usr/local/bin`.
    4. Build one root helper that option D of
       `plans/follow-ups/boat-provision-retry-inputs.md` can reuse. That
       option needs the same root for the real Boat provisioning.
    5. Keep the existing cases unchanged. The case with the missing helper
       file deletes the installed helper after the install step.
    6. See a case fail on an assertion when Boat mode installs
       `runner-entry.sh` at the guest's path.
  - **B.** Use the installed files of A in the real guest fixture of
    `image/check-runner-environment.py`. Start the guest at its default
    entry with a fixture registration and listener, and check the
    filtered journal output. Substitute only unrelated provider steps and
    external services. B repeats the delivery case of the runner check and
    the environment case of the guest check. It needs a built guest and a
    completion signal through `runuser`. Effort: Medium.
  - **C.** Add a registered restore check for each assembled image before
    release. It covers the same link on the provider, but it needs that
    infrastructure, and it gives no local regression. A release check is a
    new gate, which needs the user. Effort: Large.
  - **D.** Move the four Boat runner install lines of `image/provision.sh`
    into one script that takes an optional root, as
    `image/boat-build-locale.sh` does. `provision.sh` calls it in Boat
    mode, and the bundle carries it. `image/check-boat-runner.py` installs
    through that script, not through its own table. Cost: it changes the
    shared provisioning script, the inputs of build step `086` at
    `image/boat_build_bundle.py:77-81`, and the build guide. The call in
    `provision.sh` keeps no local check. So a stray install line there
    still passes until the restore smoke. Effort: Medium.
- **Left out:**
  - A copy of the Boat paths in a second guest fixture, because it keeps a
    second install map.
  - A check of the shell text, because a moved line, or a file that fails
    to start, still passes.
  - No change, with the restore smoke as the only check, because a break
    then shows only after a paid build and restore.
  - The entry step of the restore smoke inside the build, before the
    snapshot, because it adds a build error. That is a new gate, which
    needs the user. It also runs only after most of the paid build.
  - The image checks in the repository gate, because that is a new gate.
    `plans/follow-ups/image-checks-in-gate.md` holds that choice.
  - A real guest start on the registered path of the launch fixture,
    because it copies the guest fixture of
    `image/check-runner-environment.py` with more stubs. A started guest
    also deletes the delivered file that this path checks.
  - A separate wrapper path that the guest unit names with
    `--runner-entry`, because it moves the copy of the path into the unit.
    A missing wrapper then stops each Boat job.
  - A change of the guest in Rust, because the guest already starts its
    fixed default path.
- **Wider fix:** A. It takes the installed files from the real build path.
  D is structural, but it leaves the call in `provision.sh` unchecked, so
  a stray install line still passes. The guest part of B adds no
  protection against the class.
- **Previous examples:** Commit `92f34012` made the locale check run the
  real Boat provision script, to catch a missing installer call. Commit
  `eef260e7` added the real guest check,
  `image/check-runner-environment.py`.
- **Related:**
  - `plans/follow-ups/boat-provision-retry-inputs.md`, a shared fixture.
    Its option D needs the same private root for the real Boat
    provisioning. Build that root once, in the item that lands first, and
    reuse it in the other.
  - `plans/follow-ups/boat-stderr-safe-reasons.md`, the same cause for its
    option B. That option adds one more copy of the entry paths, in the
    journal helper. If a branch takes it, do A first, so that its
    regression runs on the installed paths. Its recommended option D
    needs no path.
  - `plans/follow-ups/boat-guest-registration-path-coverage.md`, the same
    class in the launch check: a fixture copies a product path. A does not
    cover it, because that path lives in the guest unit. Fix them apart.
  - `plans/follow-ups/boat-runner-stderr-drain.md`, a shared trigger for
    B. The guest in `image/check-runner-environment.py` sends its exit
    report to `https://127.0.0.1:1/unused`, which fails at once. So the
    journal check of B needs a completion signal through the guest and
    `runuser`.
- **Recommendation:** A, the orchestrator's call on 2026-11-21, changed
  from B. A runs the real Boat branch of `provision.sh` and catches the
  named mistake at the lowest level, with no image change. No realistic
  break is known that B catches and A does not. D is the structural choice
  for one install map, but it leaves the call in `provision.sh` unchecked.
- **Action:** fix after merge, apply A, because the restore smoke checks
  the installed entry before publication.
