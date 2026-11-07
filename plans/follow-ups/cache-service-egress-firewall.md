# The cache egress firewall needs live proof

- **Severity:** High. **Effort:** Medium.
  **Confidence:** Medium. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-06, from plan item 85, option E, which the user chose
  as a follow-up item.
- **Source:** Review.
- **Category:** Security.
- **Location:** the cache service of the agent,
  `crates/ciproxy-agent/src/cache_service.rs` and
  `crates/ciproxy-agent/src/cache_network.rs`.
- **Reason:** The service-user firewall is implemented. Its real kernel,
  service restart, fallback, and boot proof still need a disposable KVM
  host. The implementation session has no usable KVM device.

- **Terms:**
  - **Machine cache:** the host service that stores cache entries and sends
    the cache requests of each job to GitHub.
  - **Handoff:** the message from the runner hook in a job VM to the agent
    with the original job data: the cache URLs, the token, and the rights.
- **Background:** Job code has root in its VM. The machine cache sends HTTP
  requests to the URLs of each job's handoff. Its trusted handoff rules
  keep those URLs from GitHub. The output firewall adds a separate kernel
  boundary for a future bug that admits another URL.
- **Problem:** Mocked entry-point tests prove that installation writes the
  output policy and starts the service under a separate user. They cannot
  prove real packet filtering, systemd credentials, or protection at boot.
- **Impact if no action:** An untested host difference could permit a
  private upstream request or prevent normal cache service operation.
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
- **Previous examples:** The original cache handoff now preserves each
  job's rights and URL origins in
  `crates/ciproxy-agent/src/cache_authority.rs`.
- **Recommendation:** A. Kernel enforcement covers every request of the
  cache process.
- **Action:** fix after merge, apply A. Keep this item open until the live
  proof completes. The code and local regressions alone do not finish it.

## Decision and implementation

The user selected A on 7 November 2026. The user approved its system user,
firewall before startup, blocked destinations, and resolver exception.

The implementation uses the `ciproxy-cache` system user.
Its output chain blocks host-local addresses, private IPv4 ranges,
metadata, and IPv6. The only destination exceptions are DNS to the current
resolvers and reply-direction TCP from the cache and administration ports.
Public upstream HTTPS remains available.
The root-owned loader reads `/etc/resolv.conf` before each start.
It applies the complete nftables change in one transaction.
Systemd runs it before direct starts, boot starts, and automatic restarts.
A failed policy load keeps the cache listener stopped.

Design calls on 7 November 2026:

- Keep `/var/lib/ciproxy/area/cache` root-owned. Its `data` child belongs
  to the service user. This preserves the existing host ownership checks.
- Bind only that child into `/run/ciproxy-cache/data` for the service.
  Its root start helper copies the existing administration token into a
  root-owned runtime directory with cache-group traversal and read access.
  The stop helper removes this copy. Ordinary repair does not copy it.
  This preserves the private root state directory.
- Stop a changed existing unit before it moves data. Try its old checked
  binary under the new user before downloading a release. Continue release
  repair if that protected start fails. A failed download must not leave
  the old service running as root.
- Reclaim both cache layouts before acknowledging released bytes.
  An interrupted data move can leave archives in either directory.
- Keep the inactive system account after uninstall. This avoids UID reuse.

The live entry points are:

- `crates/ciproxy-agent/src/cache_network.rs`: apply the account and loader
  before managed cache starts.
- `crates/ciproxy-agent/src/cache_unit.rs`: install the UID policy, loader,
  private data mount, credential, and systemd unit.
- `crates/ciproxy-agent/src/cache_service.rs`: protect an installed cache
  before release download and restart it when needed.
- `crates/ciproxy-agent/src/cache_admin.rs`: reclaim both data layouts.
- `crates/ciproxy-agent/scripts/smoke-cache-egress.py`: run the required
  two-phase proof against the installed service and real GitHub jobs.

## Remaining live proof

Use root on a disposable registered KVM host with the matching agent,
cache-sidecar, image, and staging gateway. The host needs systemd, nftables,
iproute2, Python 3, and `setpriv`. Its allowed private repository must have
the checked-in smoke workflow with the pinned official `actions/cache`.
The staging gateway must route its jobs to this machine.
Wait for every existing job. Keep other workflows and host repairs stopped.

Create the private input and credential files used by the own-machine smoke
client. The input needs `public_url`, `workspace_id`, `machine_id`,
`repository`, `ref`, and `cache_firewall_disposable: true`.
The credential file needs `github_token` and `session_cookie`.
Pass the administration token only through its private host file path.

Run from the repository root:

```sh
export CIPROXY_OWN_SMOKE_INPUT=/private/own-smoke-input.json
export CIPROXY_SMOKE_CREDENTIALS=/private/own-smoke-credentials.json
export CIPROXY_CACHE_ADMIN_TOKEN_FILE=/var/lib/ciproxy/cache-admin-token
python3 crates/ciproxy-agent/scripts/smoke-cache-egress.py
```

The first phase must finish its network, real-job, restart, failed-start,
and fallback checks. It returns SKIP with exit 77 because boot proof remains.
After it restores temporary changes, reboot the same disposable host:

```sh
sudo systemctl reboot
```

Reconnect. Set the same three private file paths again.
Run from the same checkout after the host boots:

```sh
python3 crates/ciproxy-agent/scripts/smoke-cache-egress.py --after-boot
```

Require PASS with exit 0 from the second phase.
It must observe a changed boot ID and repeat the network and real-job
checks. Missing prerequisites or a missing reboot returns SKIP with exit
77. A failed assertion returns exit 1. SKIP is not proof.
The script restores temporary policies, services, links, routes, sessions,
and marker files on normal failure or operator interruption.
It uses only public probe data and never prints credentials.

The proof must cover:

- Reachable loopback, private IPv4, metadata, public host-local, IPv6, and
  IPv4-mapped loopback targets. Cache requests must be blocked by the
  installed output policy without reaching the controlled listeners.
- Original-direction established connections. They must remain blocked
  after the installed policy replaces a temporary test exception.
- DNS over UDP and TCP to each configured resolver as the cache UID.
- Public upstream HTTPS, private token access through systemd, unchanged
  root control permissions, and distinct real VMs with a cache hit.
- Inbound interface protection, direct start with the agent stopped and
  policy absent, automatic restart, and a listener closed on policy error.
- A successful real GitHub cache restore while firewall setup prevents the
  machine cache from starting.
- A real reboot, followed by the repeated network and real-job checks.
