# Boat build replays fail on reused installer files

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-20, from `conductor/m27a-boat-native-image`, review
  round 1, the wider fix of the PyPy resume finding.
- **Location:** `image/boat-build-execute.py` (the replay decision),
  `image/boat_build_native.py` (the native adaptations of upstream
  scripts), and these rendered upstream installers:
  `install-dotnetcore-sdk.sh` (`ln -s /usr/share/dotnet/dotnet
  /usr/bin/dotnet`), `install-java-tools.sh` (`ln -s` of each JDK `x64`
  link, of `/usr/bin/mvn` and of `/usr/bin/gradle`), `install-swift.sh`
  (three `ln -s` lines), `install-google-chrome.sh` (`ln -s` of
  chromedriver and Chromium), and `install-nvm.sh` (`mkdir $NVM_DIR`).
- **Reason:** The branch made PyPy safe to replay. The other installers
  need their own native changes, and a refusal of unsafe replays is a new
  rejection, which the always-on rules reserve for the user.

- **Terms:**
  - **Native Boat build:** the build that installs the pinned image
    definition directly on a Boat sandbox, step by step, with
    `image/boat-build.py` and `image/boat-build-execute.py`.
  - **Resume and refresh:** the two commands that run the native build
    again on the same sandbox after a failed step or after a change of the
    build inputs. They keep each passed step whose identity, and the
    identity of every earlier step, did not change.
  - **Replay:** a run of an upstream installer on a filesystem where an
    earlier run of the same installer already left files.
  - **Native adaptation:** a change that the bundle makes to the rendered
    upstream scripts for the Boat build only, in
    `image/boat_build_native.py`, with a recorded reason.
- **Background:** The upstream installers of `actions/runner-images` run
  once on a fresh VM. The native Boat build can run an installer again on
  the same sandbox: after a failure of that step, or after a change of an
  earlier step. Several installers then fail, or pass with wrong files,
  because of files that their own earlier run made.
- **Problem:**
  - On 2026-11-20 a live Boat build ran `install-dotnetcore-sdk.sh` a
    second time after a failed test. It stopped with `ln: failed to create
    symbolic link '/usr/bin/dotnet': File exists`, because the first run
    had made that link.
  - A replay of `install-java-tools.sh` stops at its `ln -s` to
    `/usr/bin/mvn` with `File exists`. Its `ln -s` of an existing JDK `x64`
    link exits 0 and adds a link inside the JDK folder that points to the
    folder itself.
  - Replays of `install-swift.sh`, `install-google-chrome.sh`, and
    `install-nvm.sh` stop with `File exists`.
- **Impact if no action:** After a failed step or a changed input, the
  operator must repair files by hand or start a new build from a clean Boat
  template, which costs about an hour and a paid sandbox.
- **Root question:** How should resume and refresh run an upstream
  installer again on a sandbox where its earlier run left files, with no
  refusal of the replay?
- **Options:**
  - **A.** Add a native adaptation that makes each installer replace the
    links and the version folders that it owns: rewrite its plain `ln -s`
    lines to `ln -sfn`, and remove its own incomplete version folder before
    it installs again, as the branch did for PyPy. Add a replay regression
    for each changed installer. Effort: Medium.
  - **B.** Give each replay-unsafe installer a list of the paths that it
    makes, and remove those paths before the executor runs the step again.
    Each list must follow the upstream pins. Effort: Large.
  - **C.** Save a Boat checkpoint before each step and restore it before a
    retry. This needs Boat snapshot operations for each step and changes
    the build lifecycle. Effort: Large.
  - **D.** Document only that a replay failure that comes from an
    installer's own earlier files needs a new build from a clean template.
    Effort: Small.
- **Left out:** A refusal of unsafe replays, because it is a new build
  rejection, which the always-on rules reserve for the user.
  A shell wrapper that makes `mkdir` and `ln -s` accept existing paths,
  because it cannot repair a `mv` into an existing folder.
- **Wider fix:** A. It changes the rendered commands by one rule, so a new
  installer with the same commands is also safe.
- **Previous examples:** The branch already made the native setup safe to
  repeat: `mkdir -p` for `/imagegeneration`, `ln -sfn` for the test helper
  link, and merged folder moves. It also made the PyPy installer replace
  its incomplete version folders.
- **Recommendation:** A. It keeps resume useful for the common failures and
  adds no refusal.
- **Action:** fix after merge, apply A, because the branch needed only the
  PyPy change for its review finding and the other installers need their
  own regressions.
