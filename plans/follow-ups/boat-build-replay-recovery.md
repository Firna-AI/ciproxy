# Boat build replays fail on reused installer files

- **Severity:** Medium. **Likelihood:** Common. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-20, from `conductor/m27a-boat-native-image`, review
  round 1, the wider fix of the PyPy resume finding. Review round 2
  confirmed the class and added facts and options.
- **Location:** `image/boat-build-execute.py` (the replay decision),
  `image/boat_build_native.py` (the native adaptations of upstream
  scripts), and these rendered upstream installers:
  `install-dotnetcore-sdk.sh` (`ln -s /usr/share/dotnet/dotnet
  /usr/bin/dotnet`), `install-java-tools.sh` (`ln -s` of each JDK `x64`
  link, of `/usr/bin/mvn` and of `/usr/bin/gradle`), `install-swift.sh`
  (three `ln -s` lines), `install-google-chrome.sh` (`ln -s` of
  chromedriver and Chromium), and `install-nvm.sh` (`mkdir $NVM_DIR`).
  Also plain `ln -s` in `install-julia.sh`, `install-miniconda.sh`,
  `install-microsoft-edge.sh`, `install-mssql-tools.sh`,
  `install-nodejs.sh`, `install-firefox.sh`, `install-sbt.sh` and
  `install-haskell.sh`, and `unzip` without `-o` into kept folders in
  `install-java-tools.sh`, `install-google-chrome.sh`,
  `install-microsoft-edge.sh`, `install-ninja.sh`, `install-kotlin.sh`,
  `install-terraform.sh` and `install-sqlpackage.sh`.
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
  - The Haskell link to `/etc/skel/.ghcup` adds a link inside that folder
    on a replay, as the Java link does.
  - `unzip` without `-o` keeps the old file and exits 1 when its input is
    closed, as under `systemd-run`. So a same-version replay of Java stops
    at the Maven `unzip`, and a replay of Chrome stops at the chromedriver
    `unzip`.
  - The PyPy reuse check reads only `PyPy/<version>/x64.complete`. After a
    refresh with a new PyPy pin for the same Python version, it keeps the
    old PyPy build. A new Python version leaves the old version folder.
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
  - **E.** The executor records the new top-level entries of each step in
    a fixed list of install folders, such as `/usr/bin`, `/usr/local`,
    `/usr/share`, `/opt`, `/etc/skel` and the tool cache. Before a replay,
    it removes the recorded entries of each step that runs again, in
    reverse step order, and keeps entries that a package owns. The
    installers stay unchanged. It does not undo an edit inside an existing
    file, and a wrong folder list can remove a provider path. Effort:
    Large.
  - **F.** One rendering rule for every installer: plain `ln -s` becomes
    `ln -sfn`, plain `mkdir` becomes `mkdir -p`, and each `unzip` gets
    `-o`, with a separate edit for each `mv` into a kept folder. It can
    replace a file of Boat's base on a first run, and it keeps old version
    folders after a pin change. Effort: Medium.
- **Left out:** A refusal of unsafe replays, because it is a new build
  rejection, which the always-on rules reserve for the user.
  A shell wrapper that makes `mkdir` and `ln -s` accept existing paths,
  because it cannot repair a `mv` into an existing folder.
- **Wider fix:** E. It does not depend on the commands of an installer, so
  it still covers a new installer or a changed upstream line.
- **Previous examples:** The branch already made the native setup safe to
  repeat: `mkdir -p` for `/imagegeneration`, `ln -sfn` for the test helper
  link, and merged folder moves. It also made the PyPy installer replace
  its incomplete version folders.
- **Recommendation:** E. It removes the earlier outputs of every replayed
  step with no edit for each installer line. A and F must know each line
  and can keep an old version after a refresh with new pins.
- **Action:** fix after merge, apply E, because it is a Large change of the
  replay path that this branch does not need for a clean build.
