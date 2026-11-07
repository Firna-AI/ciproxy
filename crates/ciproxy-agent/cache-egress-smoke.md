# Cache output firewall proof

Use a disposable registered x64 Linux KVM host with matching agent and
cache-sidecar releases. Run from the repository root as root.
The host needs systemd, nftables, iproute2, Python 3, and `setpriv`.
The agent's host preparation supplies the production firewall tools.

Prepare the private repository, workflow, staging App, image, and private
input files from [the runtime smoke guide](smoke-checks.md).
The workflow needs its `cache` mode with the pinned official `actions/cache`.
The staging gateway must route those jobs to this disposable machine.
Wait for every existing job before each phase.
Add `"cache_firewall_disposable": true` to the private input JSON.
This confirms that the script can stop the cache and agent temporarily.
Keep other workflows and host repair operations stopped during this proof.

Set only private file paths for credentials:

```sh
export CIPROXY_OWN_SMOKE_INPUT=/private/own-smoke-input.json
export CIPROXY_SMOKE_CREDENTIALS=/private/own-smoke-credentials.json
export CIPROXY_CACHE_ADMIN_TOKEN_FILE=/var/lib/ciproxy/cache-admin-token
python3 crates/ciproxy-agent/scripts/smoke-cache-egress.py
```

The first phase returns exit 77 after its checks pass.
It writes a checkpoint with the current boot ID.
Reboot the same disposable host after all temporary changes are restored:

```sh
sudo systemctl reboot
```

Reconnect to the host. Set the three private file paths again.
Run the second phase from the same checkout:

```sh
python3 crates/ciproxy-agent/scripts/smoke-cache-egress.py --after-boot
```

Only this second phase can return PASS with exit 0.
It requires a different boot ID and an enabled, active cache service.
Missing root, KVM, tools, private files, or the reboot reports SKIP and
exit 77. A failed assertion returns exit 1.
A skipped check never counts as live success.

## What the script checks

The script calls the installed administration and job listeners.
It registers temporary sessions with empty tokens and public probe data.
It sends their upstream requests through the actual cache process.
It never prints a token, response body, or private input.

Temporary HTTP listeners supply reachable loopback, private IPv4, metadata,
IPv6, IPv4-mapped loopback, and public host-local destinations.
The control UID must reach each listener first.
The cache must return a failed upstream response without reaching it.
The installed firewall's drop counter must increase for that request.
A temporary namespace also tests the existing inbound-interface block.
The script creates a private connection as the cache UID under a temporary
test exception. It reloads the installed policy before that connection
sends its request. The original-direction request must then be blocked.

The script uses `10.254.253.0/30`, `169.254.169.254/32`,
`93.184.215.254/32`, and `2001:db8:cf::/64` only for controlled listeners.
It adds them to temporary links and removes them afterwards.
It adds a temporary host route for the controlled metadata address.
An existing exact metadata route makes setup fail without replacing it.
Do not use this script on a host that depends on these probe addresses.

The cache UID must get DNS replies from each configured resolver over both
UDP and TCP. A public HTTPS request must pass through the cache.
Real GitHub jobs must save and restore an entry through distinct job VMs.
The second job must report a machine-cache hit.
These checks also prove data writes, VM replies, and administration replies.

The first phase starts the cache with the agent stopped and its owned
firewall table absent. The unit must restore protection before it listens.
It checks a legacy data move with a public marker.
It kills the cache process and checks the automatic restart.
An invalid policy must keep both the process and listener stopped.
A real GitHub job must restore from GitHub during that firewall failure.
The second phase repeats the network and real-job checks after a real boot.

Each mutation has cleanup for normal failures and operator interruption.
The script restores the policy, services, routes, links, temporary sessions,
and marker. It keeps the public smoke cache entry for the restore check.
Evidence stays under `.context/agent-follow-ups/firewall/host-proof.json`
and the private-file smoke client's `.context/ciproxy-mvp/own-smoke-*` paths.
The script writes no credentials into that evidence.
