# Repository gate skips the image checks

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-20, from `conductor/m27a-boat-native-image`, review
  round 1, a problem outside the scope that the check of the prepare
  argument finding raised.
- **Location:** `xtask/src/check/phases.rs` (the gate phases),
  `.github/workflows/ci.yml` (the CI job), `image/checks.md` (the hand
  commands), and the checks `image/check-boat-build.py`,
  `image/check-boat-delete.py`, `image/check-boat-metadata.py`,
  `image/check-boat-run.py`, `image/check-docker-attach.py`,
  `image/check-export.py`, `image/check-kernel-metadata.py`,
  `image/check-network-probe.py`, `image/check-runner-environment.py`,
  `image/check-template.py`, `image/macos/check-template.py`,
  `image/macos/check-archive.py`, and `image/macos/check-r2.py`.
- **Reason:** A new gate phase is a new gate, and the always-on rules
  reserve that choice for the user.

- **Terms:**
  - **Complete gate:** `cargo xtask check`, the repository check that CI
    runs for each push and pull request.
  - **Image checks:** the Python checks under `image/` that exercise the
    machine image build, the native Boat build, and the Boat launch with
    fixtures.
- **Background:** The complete gate has eleven phases: Rust format,
  TypeScript format, file length, Markdown, Rust lint, TypeScript, API
  types, Rust tests, mockups, site build, and app export. None of them
  runs an image check. `image/checks.md` gives a hand command for each
  check. Some checks need root for `chroot` (`check-export.py`,
  `check-runner-environment.py`, `check-docker-attach.py`,
  `check-boat-run.py`, `check-boat-metadata.py`). `check-boat-build.py`
  needs no root, but its package-family scenario needs `apt-get` and
  `apt-cache`, and its Ansible scenario needs `pip`.
- **Problem:** A change under `image/` can break an image check, and no
  automatic run finds it. The branch that added the native Boat build ran
  its 28 builder tests only by hand.
- **Impact if no action:** A regression in the image builder reaches the
  primary branch unnoticed until a paid live build fails.
- **Root question:** How should the repository run the image checks
  automatically?
- **Options:**
  - **A.** Add a gate phase that runs the image checks that need no root,
    first `check-boat-build.py` and `check-template.py`. CI runners have
    APT and pip. Effort: Small.
  - **B.** Add a separate CI job that runs all image checks with `sudo` on
    the GitHub-hosted runner, and keep the local gate unchanged. Effort:
    Small.
  - **C.** Keep the hand runs, and list the image checks in the
    repository's `AGENTS.md` for each change under `image/`. Effort: Small.
- **Left out:** None.
- **Wider fix:** A. It runs the checks for every change, locally and in
  CI.
- **Previous examples:** `plans/follow-ups/site-calculation-tests-in-gate.md`
  records the same class for the site calculation tests.
- **Recommendation:** A, with B for the checks that need root.
- **Action:** ask the user, because a new gate phase or CI job is a new
  gate. No gate check comes now, because a new gate needs the user's
  explicit approval. The orchestrator agent of
  `conductor/ciproxy-boat-start-ready-deadline` made this call on 22
  November 2026, under the user's delegation of that day.
