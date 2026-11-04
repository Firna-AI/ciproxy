# Writable runtime tokens bypass cache trust

- **Severity:** High. **Effort:** Large. **Confidence:** High.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-04, from PR #10, orchestrator finding 1.
- **Source:** Review.
- **Category:** security.
- **Location:** `image/runner-patch/patch-runner.py:11`,
  `crates/ciproxy-cache-sidecar/src/proxy.rs:23`, and
  `docs/protocol/cache.md`, "Trusted write-back".
- **Reason:** The alternatives change the cache trust guarantee, workflow
  transparency, or the location of privileged runtime operations.

- **Terms:**
  - **Machine cache:** Persistent cache storage on a customer's machine.
  - **GitHub cache:** The upstream cache service used by GitHub Actions.
  - **VM proxy:** The cache forwarding service inside a job VM.
  - **Runtime token:** GitHub's credential for cache and artifact calls.
  - **`pull_request_target`:** A pull request event that runs its workflow
    in the base repository's context.
- **Background:** ciproxy routes job cache calls through a VM proxy and a
  machine service. The machine applies trusted job facts to shared writes.
  The runner patch preserves the original runtime token and publishes
  original cache URLs as `ZCTIONS_*`. Job code has root access in its VM.
- **Problem:** A `pull_request_target` workflow can explicitly select
  `cache-mode: write`. GitHub then gives its runtime token permission to
  write default-branch entries. Job code can use that token and the
  original endpoints to call GitHub directly. The proxy's `can_save`
  decision cannot stop this authenticated request.
- **Impact if no action:** Untrusted pull request code can poison a
  GitHub entry that a later trusted job restores. ciproxy cannot promise
  the current unconditional trusted-write rule for direct GitHub calls.
- **Evidence:** An isolated executable model rejected the proxy save but
  accepted a direct upstream save with the same job credential. A later
  default-branch restore returned `poison`. GitHub documents the explicit
  write override and warns that it reintroduces cache poisoning.
- **Requirement:** `docs/protocol/cache.md` currently forbids shared writes
  from pull requests, including `pull_request_target`. It also requires
  transparent caching without workflow edits. No answer changes either
  requirement yet. The trusted-event list must stay unchanged meanwhile.
- **Root question:** How should ciproxy handle shared writes when a
  workflow explicitly opts into writable GitHub cache credentials?
- **Options:**
  - **A.** Require read-only GitHub cache permissions for these jobs.
    Writable credentials then cannot perform the bypass. This requires
    workflow changes and changes the transparency requirement.
    Effort: Small.
  - **B.** Keep writable runtime tokens outside the job VM. Give job code
    proxy credentials and broker its required cache, artifact, and other
    runtime operations at a trusted boundary. This preserves the stronger
    write rule but adds a privileged broker and integration work.
    Effort: Large.
  - **C.** Match GitHub's trusted-event rule in the machine cache. Only
    `push`, `workflow_dispatch`, `repository_dispatch`, `delete`,
    `registry_package`, `page_build`, and `schedule` may write
    default-branch machine entries. An explicit workflow opt-in for a
    direct GitHub write remains GitHub's rule, as on GitHub-hosted runners.
    Update "Trusted write-back" to state this boundary. This preserves
    transparency but narrows the unconditional write guarantee.
    Effort: Small.
- **Left out:** Hiding environment variables cannot protect a credential
  from job code with root access. Endpoint filtering leaves the token
  available for use elsewhere. Neither approach prevents the bypass.
- **Wider fix:** B removes writable credentials from the untrusted VM.
- **Previous examples:** The approved job-facts work records cache
  poisoning risks from job-controlled trust inputs. No earlier runtime
  credential bypass example was found.
- **Recommendation:** B best preserves the current trust guarantee.
  C is the smaller alternative when the product adopts GitHub's explicit
  workflow opt-in boundary. The user must choose this trade-off.
- **Action:** ask the user, because the options trade a privileged broker
  against workflow changes or a narrower trusted-write requirement.

Waits for the user: item 59 of plans/ciproxy-mvp.md

Sources:

- [GitHub cache permission override](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#bypassing-the-default-untrusted-trigger-cache-restriction)
- [Runner runtime credential injection](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Worker/Handlers/NodeScriptActionHandler.cs)
