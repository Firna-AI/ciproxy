# Boat provision resume lacks its runner files

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-locale-fix`, review round 1,
  item 1, of the Boat locale fix. The first pass graded it High. The
  finding check corrected the severity to Medium.
- **Location:** `image/boat-provision.sh:11-13`, `image/provision.sh:44-45`,
  `image/boat_build_bundle.py:68-77`, and `image/boat-build-execute.py:108`.
- **Reason:** A failed resume has two workarounds on the same sandbox, and
  a build that passes saves the same snapshot.

- **Terms:**
  - **Native Boat build:** the build that installs the pinned image
    definition directly on a sandbox of Boat, the overflow provider, step
    by step, with `image/boat-build.py` and `image/boat-build-execute.py`.
  - **Step identity:** a hash of a build step, of each file of its inputs,
    and of the identity of the step before it.
  - **Resume:** the build command that runs the failed step and each later
    step again. It reuses each passed step whose identity did not change.
  - **Refresh:** the build command that extracts a new bundle over the
    build folder and then resumes.
  - **Runner files:** the four built files of the bundle in
    `/opt/ciproxy-build/artifacts`: `ciproxy-runner.tar.gz`,
    `ciproxy-worker.dll`, `ciproxy-guest`, and `ciproxy-cache-sidecar`.
- **Background:**
  - Step `085-ciproxy-artifacts` copies the runner files to `/tmp/`, at
    `image/boat_build_bundle.py:74`. Its inputs are the whole definition
    folder, the artifacts folder, and the upstream `cleanup-logs.sh`.
  - Step `086-boat-provision` runs `image/boat-provision.sh`. That script
    runs the shared `image/provision.sh` in Boat mode.
  - The shared script checks the runner archive at line 12 and installs
    the runner files at lines 13-18. In Boat mode, it deletes them at lines
    44-45 and exits.
  - Then `image/boat-provision.sh` runs `systemctl disable` and
    `systemctl daemon-reload` at lines 11-12. It runs the locale setup
    `image/boat-build-locale.sh` at line 13.
  - The machine build puts the same fixed `/tmp` names in place with Packer
    file provisioners, at `image/prepare-template.py:43-62`. So the shared
    script reads these names in both builds.
  - The executor reuses a passed step when its identity did not change, at
    `image/boat-build-execute.py:108-110`. A later step never makes an
    earlier step run again. `image/_checks_/boat_inputs.py:76` asserts
    this rule.
  - `image/boat.md:105-106` tells the operator to use `resume` when the
    cause of a failure is gone. After a fix to the image definition, the
    operator uses `refresh`.
  - The runner files stay in `/opt/ciproxy-build/artifacts` until
    finalization. So two workarounds work on the same sandbox:
    - Run `sudo cp /opt/ciproxy-build/artifacts/* /tmp/` with the Boat
      command API, then `resume`.
    - Run `refresh` with a bundle whose definition or runner files changed.
      The identity of step `085` then changes, so it copies the files
      again.
  - A late failure is rare. `localedef` needs only the `en_US` locale and
    `UTF-8` charmap sources, which the `locales` package supplies. The same
    `localedef` command passed on a sandbox from a published Boat snapshot.
- **Problem:** Step `086` deletes the runner files before its last three
  commands run. When one of them fails, a plain resume reuses step `085`,
  so nothing copies the files again. The retry then stops at the archive
  checksum at `image/provision.sh:12`.
- **Impact if no action:** After a late failure in step `086` with an
  outside cause, such as a full disk, a plain resume cannot finish the
  image. A build that passes still saves the same snapshot.
- **Root question:** How can a resume finish the image after a failure
  anywhere in the ciproxy provision step?
- **Options:**
  - **A.** Generate the locale before the runner install. A locale failure
    then leaves the runner files in place. The two `systemctl` commands
    keep their risk. Effort: Small.
  - **B.** Remove the Boat-mode deletion at `image/provision.sh:44-45`.
    Finalization already deletes the same files, at
    `image/boat-build-finalize.sh:15-18`. It edits the shared
    `provision.sh`, and it keeps the large runner archive on disk until
    finalization. Its regression needs root or a chroot. Effort: Medium.
    The first review graded it Small.
  - **C.** Give the locale setup its own final build step. A resume then
    runs only the locale setup again. The plan and the report get one more
    step, and the two `systemctl` commands keep their risk.
    Effort: Medium.
  - **D.** Move the copy of the runner files from step `085` to the start
    of `image/boat-provision.sh`. Each attempt of step `086` then copies
    them first. Add a regression that fails once and then resumes.
    Effort: Medium. The steps:
    1. First give the locale check the same isolation as the retry
       regression of step 4. For `check_locale`, `provision_locale` in
       `image/_checks_/boat_locale.py` runs the real `boat-provision.sh`
       without root. After step 2, that script copies the runner files to
       fixed `/tmp` names, which a check without root cannot keep private.
    2. In `image/boat-provision.sh`, add
       `cp "$definition/../artifacts/"* /tmp/` before the call to
       `provision.sh`.
    3. In `image/boat_build_bundle.py`, remove the line
       `cp /opt/ciproxy-build/artifacts/* /tmp/` from step `085`. Remove
       `/opt/ciproxy-build/artifacts` from its inputs. Step `086` already
       lists that folder, at line 77.
    4. Add the retry regression in a private filesystem root that holds
       its own `/tmp`. Run it as root under `chroot`, as
       `image/check-export.py` and `image/check-boat-run.py` do:
       - `create` and `provision_sandbox` in
         `image/_checks_/export_fixture.py` build a private root for the
         real `provision.sh`.
       - `copy_program` in `image/_checks_/boat_fixture.py` copies a real
         program, with its interpreter or libraries, into a root.
       - In the root, the real `boat-provision.sh` copies a fixture
         artifacts folder with the real `cp`. Stubs replace `id`,
         `useradd`, `install`, and `systemctl`.
       - A stub `provision.sh` requires the four files in `/tmp` and
         deletes them with the real `rm`, as the Boat-mode cleanup does.
       - A locale stub fails on the first attempt only.
       - The second attempt, with the same bundle, must pass. See the test
         fail on the current code first.
       - The test writes no host path, and it keeps the copy line under
         test unchanged.
    5. Add one sentence to "The ciproxy parts" in
       `image/boat-build-steps.md`: each attempt copies the runner files
       first.
  - **E.** Read the Boat inputs in place from the bundle, with no copy and
    no deletion. It changes the input lines of the shared `provision.sh`
    that the machine build also runs. Effort: Medium.
  - **F.** Let the executor copy declared inputs before each attempt of a
    step. It adds a plan field and executor code for only one step.
    Effort: Medium.
- **Left out:**
  - A fresh build or a warning in the docs, because a resume still cannot
    finish.
  - A rule that runs step `085` again with step `086`, because a later
    step must never make an earlier step run again.
  - A Boat checkpoint before each step and a restore before a retry,
    because `plans/follow-ups/boat-build-replay-recovery.md` holds it as
    its option C. That option is Large and needs Boat snapshot operations.
- **Wider fix:** D. It covers each failure point of step `086`, also a
  point that a later change adds.
- **Previous examples:** Commit `37397f61` made the PyPy installer safe to
  replay. `plans/follow-ups/boat-build-replay-recovery.md` records the
  broad class.
- **Related:** `plans/follow-ups/boat-build-replay-recovery.md` has the same
  broad class: a step that cannot run again on the state of its own
  earlier attempt. Its shape is the opposite: files that an earlier run
  left, not files that a run deleted. Its recommended option E removes
  recorded outputs before a replay, so it cannot bring back deleted files.
  Fix the two items apart.
- **Recommendation:** D. Each attempt of step `086` gets its runner files,
  and the shared `provision.sh` stays unchanged, so the machine build does
  not change.
- **Action:** fix after merge, apply D, because it is a Medium product bug
  with the likelihood Rare, and its fix gets no harder later.
