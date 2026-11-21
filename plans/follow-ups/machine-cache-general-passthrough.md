# The machine cache forwards unused non-cache requests

- **Severity:** Low. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Outside the scope.
- **Added:** 2026-11-06, from `conductor/ciproxy-mvp-milestone-13-e2e`, the
  check of item 1 in the first owner review of pull request #31. That check
  found it outside the scope.
- **Source:** Review.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-cache-sidecar/src/machine.rs:123-127`,
  `crates/ciproxy-cache-sidecar/src/machine.rs:79-101`,
  `crates/ciproxy-cache-sidecar/src/upstream.rs:27-43`, and
  `crates/ciproxy-cache-sidecar/src/proxy.rs:61-62`.
- **Reason:** The general forwarder predates the owner fixes, and its
  removal deletes an HTTP entry point, which needs the user's approval.

- **Terms:**
  - **Machine cache:** The `ciproxy-cache-sidecar machine` service on a
    customer machine. Job VMs reach its job listener on TCP port 7887.
  - **VM proxy:** The `ciproxy-cache-sidecar` proxy inside each job VM. Job
    steps send their cache and results calls to it.
  - **Session token:** The per-job token that the VM proxy sends to the
    machine cache in the `x-ciproxy-session` header.
  - **Sealed origins:** The scheme, host, and port of a job's cache and
    results URLs, which the agent saves from the first handoff.
- **Background:**
  - The VM proxy sends each cache call and each `/ciproxy/blob/` call to the
    machine cache with the session token.
  - The VM proxy sends every other request directly to GitHub, at
    `crates/ciproxy-cache-sidecar/src/proxy.rs:61-62`.
  - The machine cache still forwards each non-cache path that arrives with a
    valid session token, at
    `crates/ciproxy-cache-sidecar/src/machine.rs:123-127`.
  - That forwarder keeps the method, path, query, and body of the request,
    and it adds the job's runtime token as the bearer credential.
  - `destination` in `crates/ciproxy-cache-sidecar/src/upstream.rs:27-43`
    sends a path under `/_apis/artifactcache/` to the cache URL and every
    other path to the results URL.
  - A search of the repository found no caller that sends a non-cache
    request to the machine cache. The local smoke
    `image/runner-patch/smoke/check.py` and the sidecar fallback test send
    their results requests through the VM proxy, which goes to GitHub.
  - A replay of a handoff cannot change the sealed origins. So the forwarder
    reaches only the sealed results origin of the job.
  - The forwarder is part of the HTTP interface of the machine cache. It is
    an entry point, so it is functionality, also with no caller.
- **Problem:** The machine cache keeps a general forwarder that no caller in
  the repository uses. A job can make the host send any method, path, and
  query to its results origin, with the job's runtime token.
- **Impact if no action:** The host keeps request surface that no product
  path needs. A later bug in the origin seal would widen what a job can
  reach through it.
- **Root question:** Which requests must the machine cache forward upstream
  for a job?
- **Options:**
  - **A.** Remove the general forwarder, so that the job listener of the
    machine cache refuses each non-cache path instead of forwarding it. Keep
    the direct GitHub route of the VM proxy. Repeat the caller search first.
    Effort: Small.
  - **B.** Keep the forwarder. The sealed origins, the first-handoff rule,
    and the deferred egress firewall bound what it can reach.
    Effort: Small.
- **Left out:** An allowlist of non-cache routes, because no caller needs
  any non-cache route, so A is better on every point. Removing the direct
  GitHub route of the VM proxy fails the goal, because job steps need their
  artifact, log, and other results calls.
- **Wider fix:** None. The egress firewall follow-up already bounds every
  destination of the machine cache. This item only removes an unused route.
- **Previous examples:** None found.
- **Related:** The [cache egress firewall](cache-service-egress-firewall.md)
  item is the network backstop for every URL that the machine cache
  reaches. This item only removes an unused route. Neither item waits for
  the other.
- **Recommendation:** A. It removes host request surface that no caller
  uses.
- **Action:** ask the user, because the forwarder is an HTTP entry point,
  and its removal deletes functionality, which needs the user's approval.
