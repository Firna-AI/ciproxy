# The machine cache can reach private host addresses

- **Severity:** High. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from plan item 85, option E, which the user chose
  as a follow-up item.
- **Source:** Review.
- **Category:** Security.
- **Location:** the cache service of the agent,
  `crates/ciproxy-agent/src/cache_service.rs` and
  `crates/ciproxy-agent/src/cache_network.rs`, after pull request #31.
- **Reason:** Item 85 = F closes the known path, a job that chooses the
  first cache handoff. This item protects the host against any other URL
  that a job can supply.

- **Terms:**
  - **Machine cache:** the host service that stores cache entries and sends
    the cache requests of each job to GitHub.
  - **Handoff:** the message from the runner hook in a job VM to the agent
    with the original job data: the cache URLs, the token, and the rights.
- **Background:** Job code has root in its VM. The machine cache on the host
  sends HTTP requests to the URLs of each job's handoff, and it follows the
  download links that GitHub returns. The fixes of #31 finding 1 and item 85
  make sure that the URLs come from GitHub's real handoff.
- **Problem:** Nothing at the network level stops the cache service from
  reaching host-local or private addresses, such as the cloud metadata
  service. A future bug that lets a job pick a URL again would expose them.
- **Impact if no action:** One more bug of this kind could let a job read
  the host's cloud credentials through the machine cache.
- **Root question:** How does the host stop the cache service from reaching
  private addresses, whatever URL it gets?
- **Options:**
  - **A.** Run the cache service as its own system user. Before the service
    starts, install a kernel firewall output rule for that user that drops
    host-local, private, and IPv6 destinations, and allows only DNS to the
    resolver. Prove it on the KVM host. Effort: Medium.
  - **B.** Validate each upstream and download address in the cache
    service: resolve the name, check that the address is public, and pin
    it for the request. Effort: Medium.
- **Left out:** Cutting the cache service off from all private networks,
  because it needs routing changes for the VM connections, at Large effort.
- **Wider fix:** A. The kernel enforces it for every request of the service.
- **Previous examples:** #31 finding 1 and plan item 85.
- **Recommendation:** A. The user chose it as option E of item 85.
- **Action:** fix after merge, apply A, because the user chose it as a
  follow-up item of item 85.
