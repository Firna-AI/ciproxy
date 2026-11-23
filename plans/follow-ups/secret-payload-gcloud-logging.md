# Secret Manager payloads can reach Cloud SDK logs

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/h3-macos-release-from-r2-qjch`, review round 1, findings 1
  and 4.
- **Location:** `deploy/scripts/deploy_support.py`, `sync_release_secret`;
  `deploy/scripts/optional_secret.py`, `QUIET_GCLOUD`;
  `deploy/runtime.md`, the `gcloud secrets versions add` steps of the
  session key, the Boat snapshot, and the R2 images key;
  `deploy/setup.md`, the release token step;
  `deploy/scripts/register-cdn-key.sh`; `deploy/operations.md`, the key
  rotation step.
- **Reason:** The shared wrapper also changes the release-token sync and
  four operator steps that are older than the branch that found this.

- **Terms:**
  - **Cloud SDK:** the Google Cloud command-line tools that give `gcloud`.
    CI pins version 592.0.0.
  - **`core/log_http`:** a Cloud SDK property. When it is on, `gcloud`
    prints each HTTP request and response body, and it also writes each
    console line to its own log file under the SDK configuration folder.
- **Background:** Deploy reads Secret Manager payloads with
  `gcloud secrets versions access latest` and copies them into
  Kubernetes Secrets over standard input. Operators add versions with
  `gcloud secrets versions add --data-file=-`. In SDK 592.0.0, the body
  handlers of `core/log_http` are in `googlecloudsdk/core/transport.py`,
  and the log-file copy is in `googlecloudsdk/core/log.py`. Environment
  variables override the configuration file
  (`googlecloudsdk/core/properties/parsers.py`).
- **Problem:** The R2 images key read in `optional_secret.py` and the R2
  setup command in `deploy/runtime.md` now set
  `CLOUDSDK_CORE_LOG_HTTP=false` and
  `CLOUDSDK_CORE_DISABLE_FILE_LOGGING=true` for their own `gcloud` call.
  Every other payload read and write still runs with the caller's
  settings: the release-token sync in `deploy_support.py`, which does not
  capture stderr, so its response body would reach the public CI log; the
  session key, Boat share code, and release token steps of the guides;
  `register-cdn-key.sh`; and the CDN key rotation in `operations.md`. Each
  setup program is also a separate snippet in a guide, and no test runs
  them. The R2 setup snippet once stopped at its first prompt, because
  `input()` read the here-document that held the program.
- **Impact if no action:** A diagnostic run with `core/log_http` on can
  write a usable secret to a terminal, a CI log, or an SDK log file.
- **Root question:** How do all deploy scripts and operator steps move a
  Secret Manager payload without recording it?
- **Options:**
  - **A.** Add the two settings to each remaining `gcloud` payload call
    and snippet. Effort: Small.
  - **B.** Add one shared command wrapper for Secret Manager payload reads
    and writes with those settings, and move each caller to it. Effort:
    Medium.
  - **C.** B, plus one tested operator command that adds a version of
    each optional secret from hidden prompts, so the guides show one line
    each. Test it through the fake `gcloud` in
    `deploy/tests/fake_promotion_tools.py`. Effort: Medium.
  - **D.** Encrypt each payload with an asymmetric Cloud KMS key before
    any tool sees it, and decrypt it in the runtime. Effort: Large.
- **Left out:** Terminal-only suppression: SDK file logging stays on. Job
  environment defaults: a diagnostic run overrides them, and they do not
  reach operators. A throwaway `CLOUDSDK_CONFIG`: the operator's
  credentials live there.
- **Wider fix:** D removes the class, because only ciphertext moves. C
  puts the rule in one tested place.
- **Previous examples:** The Boat snapshot delivery, commit `698cd176`,
  used stream capture only.
- **Recommendation:** C. One wrapper and one tested setup command keep
  the settings in one place and catch a broken prompt.
- **Action:** fix after merge, apply C, because it reaches deploy paths
  and guides that are older than the branch that found it.
