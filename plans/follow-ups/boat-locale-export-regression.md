# Locale check misses a dropped LOCPATH export

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, final review
  round 4, finding 9.
- **Location:** `image/_checks_/boat_locale.py:57`, `:97`, and `:99-104`,
  `image/boat-build-locale.sh:24`, and `image/boat-build-checks.md:96-99`.
- **Reason:** The branch that found it used both of its actual fix rounds,
  and this Medium test finding is not critical.

- **Terms:**
  - **Boat's controller:** Boat's agent in a restored sandbox. It runs
    backend commands as the account with `HOME=/home/user`.
  - **Profile:** `/etc/profile.d/ciproxy-boat-locale.sh`, which
    `image/boat-build-locale.sh` installs and a login shell reads.
  - **`LOCPATH`:** the environment variable that tells the C library where
    to find locale data.
  - **Export:** the shell step that passes a variable to child processes.
    A variable that is not exported stays inside the shell.
  - **Probe:** the command `/usr/bin/locale charmap`. The restore check
    sends it, and the local check runs it.
- **Background:**
  - The requirement, at `docs/protocol/agent-images.md:148-151`: from the
    first command after a restore, a command of Boat's controller has the
    UTF-8 locale of Boat's base system. The controller adds no locale
    warning to its stderr.
  - For `HOME=/home/user`, the profile adds the dedicated locale folder to
    `LOCPATH` (guard at `image/boat-build-locale.sh:15`). It exports
    `LOCPATH` at line 24. So the commands of Boat's controller find the
    `en_US.UTF-8` data while Boat rebuilds its standard locale files
    (`image/boat-build-steps.md:129-134`).
  - Without that data, a command in that gap prints a `setlocale` warning.
    The pinned Boat client rejects each helper reply with text on stderr
    (`image/boat-build-steps.md:121-124`).
  - The profile exports `LOCPATH` today. Its installed text has not
    changed since commit `5a493ed6`, so the live `en_US.UTF-8` restore
    evidence stays valid.
  - `check_locale` in `image/_checks_/boat_locale.py` runs the real
    `boat-provision.sh`. `read_profile`, at line 49, sources the profile
    twice. After each read, it prints the shell value of `LOCPATH`, at
    line 57. It also captures the child environment with `env -0`.
  - For the controller read, line 97 keeps only the printed shell values
    and drops the captured child environment. Lines 99-104 then set the
    probe's `LOCPATH` from that shell value. So the probe never inherits
    `LOCPATH` from the profile.
  - The image checks run only by hand. The 11 phases of the repository
    gate do not run them (`xtask/src/check/phases.rs:33-46`).
- **Problem:** If a later edit drops only `export LOCPATH` from
  `image/boat-build-locale.sh:24`, each assertion of `check_locale` and
  `check_locale_smoke` still passes. A read of the code shows this. Nobody
  has run it.
- **Impact if no action:** A later edit can drop the export while the local
  check still passes. Then a command of Boat's controller in the gap prints
  a locale warning, the pinned client rejects the reply, and the runner
  does not start. No current image has this defect.
- **Root question:** How does the test prove that a controller command
  inherits the locale path?
- **Options:**
  - **A.** Assert that the exported environment contains the expected
    `LOCPATH`. Keep the current probe inputs. Effort: Small.
  - **B.** Build the probe paths from the exported environment that the
    test already captures. Keep the existing path and command assertions.
    Effort: Small. The steps:
    1. After the controller read, take `LOCPATH` from the child
       environment that `read_profile` captures. Treat a missing exported
       `LOCPATH` as unset. Never fall back to the shell value.
    2. Map that value to the fixture folders for the probe, as lines
       99-104 do today.
    3. Keep the real provisioner run and every current assertion: UTF-8,
       empty stderr, the unique alias, `C.UTF-8`, the runner case, the
       inherited path, and the real ASCII probe of `check_locale_smoke`.
    4. Prove it once: remove `export LOCPATH`, see the check fail, and
       restore the line. Without `LOCPATH`, no lookup reaches the fixture
       folder, so the unique-alias case fails on each host.
    5. Update `image/boat-build-checks.md:96-99`, which says that the
       check sets `LOCPATH` itself.
  - **C.** Add a test that runs the profile and the command together in a
    private filesystem root. This needs a privileged test fixture. Effort:
    Medium.
  - **D.** Make `read_profile` also record the `LOCPATH` that a child
    process receives after each read. Each caller checks that value
    against the same fixed values as the shell value. Then the controller,
    inherited, and runner reads all check the export. Each caller gets one
    more list to check, and today only the controller read can fail on it.
    Effort: Small.
  - **E.** Do D for each locale variable: `LANG`, `LANGUAGE`, each `LC_*`
    name, and `LOCPATH`. A later edit that sets any locale variable with no
    export then fails. It is a test for a whole class of defect, so it is
    blocked until the user approves it. Nobody has asked for that approval.
    Effort: Small.
- **Left out:**
  - A check of the shell variable only, because it does not prove that a
    child gets the value.
  - The live smoke alone, because it needs a paid sandbox and gives no
    local regression check.
  - A text search for `export LOCPATH`, because a moved export or an export
    that never runs still passes it.
  - An edit of the profile, because the export is present, and an edit
    changes the snapshot.
  - A new gate phase for the image checks, because the user decides each
    new gate.
- **Wider fix:** E, which is blocked. B is a direct fix, because it must
  know that `LOCPATH` carries the path to the probe.
- **Previous examples:** Commit `eef260e7` made a check observe the
  environment that the runner process receives, through a stub listener at
  `image/check-runner-environment.py:77-80`.
- **Related:** `plans/follow-ups/boat-provision-retry-inputs.md`. Step 1 of
  its option D moves `check_locale` into a private root under `chroot`.
  When that lands, remove the fixture path mapping of B, instead of keeping
  two fixtures.
- **Recommendation:** B. The UTF-8, empty-stderr, and alias assertions then
  use the `LOCPATH` that a child receives, and the fixture still needs no
  root. A and D also catch the defect. C needs root, and B already fails at
  the lowest level.
- **Action:** fix after merge, apply B, because the branch that found it
  used both of its actual fix rounds, and the finding is not critical.
