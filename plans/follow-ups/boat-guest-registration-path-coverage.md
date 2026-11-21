# Launch check ignores the guest registration path

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-21, from `conductor/boat-runner-start`, review round
  2, a problem outside the scope that the check of finding 7 raised.
- **Source:** Review.
- **Category:** Test.
- **Location:** `image/_checks_/boat_fixture.py:76-84`,
  `image/ciproxy-boat-guest.service:10`, `image/boat-run.sh:6-9`,
  `image/check-boat-run.py:86-99`, and `image/boat-smoke.md:102-103`.
- **Reason:** The fixture came from an earlier branch, and the unit path is
  correct today, so only a later change can slip through.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Guest:** the ciproxy program in the image that reads the delivered
    registration and starts the runner of one job.
  - **Guest unit:** `image/ciproxy-boat-guest.service`, the systemd unit
    that starts the guest in a Boat sandbox.
  - **Launch script:** `image/boat-run.sh`, which puts the delivered
    registration at a root-only path and queues the start of the guest.
  - **Restore smoke:** the manual check in "Restore without a
    registration" of `image/boat-smoke.md`, on a sandbox restored from a
    new Boat snapshot.
- **Background:**
  - The orchestrator's call on 2026-11-21: record this gap outside the
    scope of the branch that found it, with no fix there. The fixture
    lines came from commit `873f47c2` on 2026-11-20.
  - The launch script installs the delivered file at
    `/run/ciproxy/registration.json`, with mode 0600 and root as its owner,
    at `image/boat-run.sh:6-9`. The guest unit passes the same path with
    `--registration-file`, at `image/ciproxy-boat-guest.service:10`. The
    flag has no default, at `crates/ciproxy-guest/src/cli.rs:42-46`. So
    today the two paths match.
  - When the guest finds no file at its path, it ends with exit status 1.
    Its error is `[ciproxy-guest] registration file is missing`, and the
    text names no path. See `crates/ciproxy-guest/src/boat.rs:35-43` and
    `crates/ciproxy-guest/src/error.rs:9`.
  - When the guest finds the file, the file must give no access to group or
    others, and the user of the guest must own it. The guest then reads the
    file, deletes it, and decodes it, at
    `crates/ciproxy-guest/src/boat.rs:46-54`.
  - `image/check-boat-run.py` runs the launch script in a private root
    with a `systemctl` spy from `image/_checks_/boat_fixture.py`. The spy
    reads the command of the start unit from
    `image/ciproxy-boat-start.service` and runs it, at lines 56-58 and 86.
    It does not read the guest unit. For the start of the guest unit, it
    does this at lines 76-84:
    - With a file at `/run/ciproxy/registration.json`, it copies that file
      to its state folder, at line 78.
    - Without one, it runs a hand-written guest command with the same
      fixed path, at lines 80-81.
  - `image/check-boat-run.py` then asserts the content, the mode 0600, and
    the root owner of the copy, at lines 86-89. Without a registration, it
    asserts exit status 1 and the missing-file error, at lines 97-99.
  - The restore smoke starts the guest unit with no registration. It
    requires `ExecMainStatus=1` and `registration file is missing`, at
    `image/boat-smoke.md:102-103`. A guest that reads a wrong path gives
    the same result.
  - A registered job finds the mismatch, as in "GitHub job smoke" of
    `image/boat-smoke.md`. The guest reports a missing file, and the job
    never starts. That run needs a full native build, a sandbox, and a
    real registration.
- **Problem:** Before a registered job, no check reads the registration
  path from the guest unit. The launch check uses its own copy of the path,
  and the restore smoke cannot tell a missing registration from a wrong
  path. So a change of the path in the guest unit or in the launch script
  passes the local checks and the restore smoke.
- **Impact if no action:** Such a change shows only in a registered job,
  after a full native build. If that job smoke is skipped, each Boat job of
  the new snapshot fails when its guest starts.
- **Root question:** How can a mismatch between where the launch puts the
  registration and where the guest reads it fail before a paid build?
- **Options:**
  - **A.** Read the command of the guest unit in
    `image/_checks_/boat_fixture.py`, as the fixture already reads the
    start unit. Use that command in place of the hand-written guest command
    at lines 80-81, with the fixture image ID. With a registration, copy
    the file from the path that its `--registration-file` names. A changed
    path in the unit or in the launch script then fails the registered
    launch case, because the copy finds no file. Effort: Small. The steps:
    1. Parse `image/ciproxy-boat-guest.service` with `configparser`, as
       lines 56-58 do for the start unit. Take the path with `shlex.split`.
    2. Set `IMAGE_ID` for the spy, as `EnvironmentFile=/etc/ciproxy/guest.env`
       does on a sandbox.
    3. Assert that the copy exists before the content assertions, so a
       mismatch fails on an assertion.
    4. See the registered case fail when the unit names another path.
  - **B.** Apply A. With a registration, also run the command of the guest
    unit with the real guest, after the copy of A. The fixture text is not
    registration JSON, so the guest checks the file, deletes it, and fails
    to decode it. Require exit status 1, no file at the path, and neither
    the missing-file error nor the private-file error. A path or permission
    mismatch then fails in the real reader of the guest. Cost: the case
    depends on a decode failure, and it asserts the absence of two error
    texts. Effort: Small.
  - **C.** Keep the path in one place. At `image/provision.sh:32`, also
    write `REGISTRATION_FILE=/run/ciproxy/registration.json` to
    `/etc/ciproxy/guest.env`, beside `IMAGE_ID`. Pass
    `--registration-file ${REGISTRATION_FILE}` in the guest unit. Read the
    same file in the launch script and in the launch fixture. Cost: it
    changes three image files and the fixture, so it needs a new native
    build. Without A, a wrong variable name still passes until a registered
    job. Effort: Medium.
- **Left out:**
  - No change, with the job smoke as the only check, because a mismatch
    then shows only after a full native build and a registered job.
  - A test that compares the path in the unit file with a literal, because
    only a new decision can fail it. It also adds a third copy of the path.
  - A restore with a dummy registration that the guest must consume,
    because B checks the same unit file before the build, at no sandbox
    cost. `image/provision.sh:30-31` installs that file unchanged.
  - A guest error that names its path, so that the restore smoke can
    compare it, because it needs a guest release. It also runs only after
    a paid build.
  - A full registered start of the real guest with a fixture listener in
    the launch check, because it repeats the guest fixture of
    `image/check-runner-environment.py`. It adds no coverage of the path
    over B.
- **Wider fix:** C. It leaves one copy of the path, so one change moves
  both readers. A and B only detect a mismatch.
- **Previous examples:** The same fixture reads the command of the start
  unit, at `image/_checks_/boat_fixture.py:56-58` and `:86`. Commit
  `873f47c2` added that pattern and the hand-written guest command.
  `plans/follow-ups/boat-runner-install-check.md` records the same class
  for the runner install map.
- **Related:** `plans/follow-ups/boat-runner-install-check.md`, the same
  class: a check copies a product path instead of reading it. Neither fix
  covers the other. Its option A installs the runner files through
  provisioning, and this item reads the guest unit. Fix them apart.
- **Recommendation:** A. It is the smallest local change that fails when
  the guest unit and the launch script disagree on the path. It changes no
  image file. C best protects the codebase, because it leaves one copy of
  the path.
- **Action:** fix after merge, apply A, because the path is correct today,
  and the fix changes only a check fixture.
