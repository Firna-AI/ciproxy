# Boat stderr filter hides safe Bash failure reasons

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Small.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-21, from `conductor/boat-runner-start`, review round
  2, finding 5. The round 1 check of the URL redaction finding first
  raised the same problem as a new problem, and no item carried it. The
  round 2 finding check confirmed it and added options D and E.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `image/boat-runner-stderr.py:24` and `:70-86`,
  `image/boat-runner-entry.sh:12`, `image/runner-entry.sh:12` and `:20`,
  `image/boat-build-steps.md:180-187`, and
  `image/boat-runner-checks.md:24-32`.
- **Reason:** The filter hides a reason only when a runner start already
  fails in an unusual setup, such as a missing runner folder.

- **Terms:**
  - **Boat sandbox:** a VM that ciproxy rents from Boat, the overflow
    provider, for one job when the customer's machines are busy.
  - **Boat entry:** the installed copy of `image/boat-runner-entry.sh` at
    `/usr/local/libexec/ciproxy-runner`, the path that the guest runs.
  - **Common entry:** the installed copy of `image/runner-entry.sh` at
    `/usr/local/libexec/ciproxy-runner-common`, which starts the listener.
  - **Journal helper:** `image/boat-runner-stderr.py`, which keeps a
    bounded, redacted copy of the runner's stderr in the sandbox journal.
  - **Error hook:** `image/boat-runner-errors.sh`, a Bash `ERR` trap that
    prints the file name, the line, and the exit status of a failed
    command.
- **Background:**
  - The orchestrator's call for this work: keep a bounded, redacted copy
    of runner stderr in the sandbox journal, with no protocol change. The
    guest, the ciproxy program that starts the runner, sends the runner's
    stderr to `/dev/null`, at `crates/ciproxy-guest/src/process.rs:83`.
    So the Boat entry is the one place that can keep it.
  - The Boat entry sends its stderr to the journal helper. Then
    `image/boat-runner-entry.sh:12` runs the common entry by its absolute
    path. So Bash starts each of its own errors in the common entry with
    `/usr/local/libexec/ciproxy-runner-common: line N:`. A probe of the
    built Boat template recorded this exact prefix on 2026-11-21.
  - A failed `exec` in the Boat entry itself starts with
    `/usr/local/libexec/ciproxy-runner: line 12:`, and then names the
    common entry path.
  - `OPAQUE` at `image/boat-runner-stderr.py:24` matches a run of 32 or
    more characters from `A-Z`, `a-z`, `0-9`, `+`, `/`, `_`, `=`, and `-`.
    The common entry path has 40 such characters in a row. The Boat entry
    path has 33.
  - `redact` at `image/boat-runner-stderr.py:70-86` first turns the
    constant missing-registration error into a fixed record, at lines
    75-76. Then it replaces the whole line with `[redacted]`, at lines
    79-82. It does so for a control character, a private word, a long
    opaque run, or a registration value. Only after that does it keep the
    text before the first `://` and replace the rest, at lines 83-85.
  - So each Bash error of either entry reaches the journal as
    `[redacted]`. A failed `cd /opt/actions-runner` at
    `image/runner-entry.sh:12` holds no private word and no registration
    value. The error hook still writes
    `[ciproxy] runner entry error: ciproxy-runner-common:12 status=1`.
  - Bash ends a non-interactive shell when `exec` fails, and the error hook
    does not run then. A local run with GNU Bash 5.2 showed this on
    2026-11-21. So after a failed `exec` of the listener at
    `image/runner-entry.sh:20`, the journal holds only `[redacted]`.
  - "The runner stderr of a Boat job" in `image/boat-build-steps.md` states
    the rule: a long opaque value drops its whole line. The public entry
    path is not opaque, but the rule matches it. The whole-line form is no
    requirement, because the URL cut already keeps part of a line.
  - The restore smoke in `image/boat-smoke.md:111-139` runs the installed
    Boat entry as the guest does, with no registration. It requires the
    stop at the registration check at `image/runner-entry.sh:19`. So on a
    snapshot that passed it, the `cd` at line 12 worked. A `cd` failure in
    a job then needs a change between the restore and the job.
- **Problem:** The opaque-value rule matches the public install paths that
  start each Bash error of the two entries. So the journal replaces the
  whole error line with `[redacted]`, also when its cause holds nothing
  private.
- **Impact if no action:** After a Bash error of an entry, the journal
  gives no cause, such as a missing or locked runner folder. After a failed
  `exec`, it keeps only `[redacted]`, so the operator must reproduce the
  failure.
- **Root question:** How can the journal keep safe startup failure reasons
  without exposing private data?
- **Options:**
  - **A.** Recognize the failed `cd` line of the common entry in the
    journal helper, and write a fixed reason without the path. Add a case
    for the absent runner folder. It covers this one failure only.
    Effort: Small.
  - **B.** Treat both installed entry paths as known public tokens. Match
    each path as a whole token anywhere in the line, and replace it with
    its file name. Check the whole original line for private words and
    registration values. Check the rest of the line for opaque runs before
    the URL cut. Add the failed `cd` regression, and keep each redaction
    assertion. Cost: the helper gets a third copy of the install paths,
    beside `image/provision.sh:20-23` and the copy table of
    `image/check-boat-runner.py`. Effort: Small.
  - **C.** Rebuild each Bash startup error from an allowlist of safe causes
    in the journal helper. It needs no exception in the opaque check, but
    it loses each cause that the list does not know. Effort: Medium.
  - **D.** Keep the whole-line drop for control characters, private words,
    and registration values, and the fixed missing-registration record.
    Then, before the URL cut, replace each word that holds a long opaque
    run with `[redacted]`. A whole word goes, so a secret with a `.` or a
    `%` in it leaves no part behind. Cost: the other words of a line with a
    long value now reach the journal, as on each line without one. A failed
    `exec` of the common entry in the Boat entry keeps no file name.
    Effort: Small. The steps:
    1. In `redact`, remove `OPAQUE.search(text)` from the whole-line check.
       Add the word rule after that check and before the URL cut. A word
       is a run of text with no space or tab.
    2. Add a case to `image/check-boat-runner.py` that runs the real Boat
       entry with passing profiles and no runner folder. Wait for the
       helper with a completion pipe, as the entry journal case does.
    3. In that case, require the `cd` reason with its path word redacted,
       and then the hook record. See the case fail on the current helper
       first. The reason line then reads:

       ```text
       [redacted] line 12: cd: /opt/actions-runner: No such file or directory
       ```

    4. In `test_stderr_retains_a_redacted_prefix_and_drains_the_burst`,
       change one expected line. The line `Redirect: example://` with the
       long fixture value becomes `Redirect: [redacted]`. Keep each
       exclusion assertion.
    5. Keep the entry journal case as it is. Its `[redacted]` line holds
       the decoded registration value, so D still drops it whole.
    6. Update the rule on long opaque values in "The runner stderr of a
       Boat job" of `image/boat-build-steps.md`. Update the stderr helper
       item of `image/boat-runner-checks.md`.
  - **E.** Install both Boat entry files at paths with no run of 32
    characters of the class, for example with a dot after `ciproxy`. Pass
    the Boat entry with `--runner-entry` in
    `image/ciproxy-boat-guest.service`. The filter stays unchanged. Cost:
    it renames public paths in the provision script, the guest unit, the
    smoke guide, the checks, and the build guide. It depends on the length
    limit of the rule, and it helps no other long value. Effort: Medium.
- **Left out:**
  - An exemption for each run that looks like a path, because a base64
    secret can look like a path. It fails the privacy part of the goal.
  - Fixed reasons in the common entry, because B is better on every point.
    The guest also drops runner stderr on machines. So the machine image
    gains nothing, and the change needs a machine release.
  - A new length limit or a character test for the opaque rule, because
    it must know the shape of the path. A long path with a capital letter
    still matches, and a long lowercase secret then passes.
  - Only a guide note that entry errors show as `[redacted]`, because the
    journal still keeps no reason.
  - An unfiltered copy for operators, because it breaks the redaction
    requirement.
  - A fix in the error hook, because Bash writes its reason before the trap
    runs, and the trap cannot read that text. A folder probe in the trap is
    A at another place.
  - A filter in the guest, because it applies the same rules at another
    place. It also needs a guest release.
- **Wider fix:** D. B must know the installed paths, so it is a direct
  fix. D works for each long public value.
- **Previous examples:** The fixed missing-registration record at
  `image/boat-runner-stderr.py:75-76` replaces one known Bash error with a
  fixed reason, as A does. Commit `88df2d91` added the URL cut at lines
  83-85, which keeps part of a line, as D does.
- **Related:**
  - `plans/follow-ups/boat-runner-install-check.md`, the same cause for B.
    The runner check copies the runner files from its own table, not
    through provisioning. So the path list of B could only be checked
    against that table. Option A of that item closes the gap. D needs no
    path, so it does not wait for that item. E changes the same install
    map.
  - `plans/follow-ups/boat-runner-stderr-drain.md`, the same file only. Its
    fix changes `forward` and `main` of the journal helper. This item
    changes `redact`. No conflict.
- **Recommendation:** D, the orchestrator's call on 2026-11-21, changed
  from B. D removes each long value as fully as B, and it keeps the rest of
  the line, as the URL cut does. It needs no list of install paths, and it
  also keeps the reason when another long public value appears. If the
  whole-line drop for long values must stay, apply B with both entry paths.
- **Action:** fix after merge, apply D, because the defect hides the reason
  only when a runner start already fails in an unusual setup.
