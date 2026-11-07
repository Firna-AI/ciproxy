# Cache and Docker integration

The agent connects each job VM to the machine cache of its host and to a
checked Docker disk. This guide gives that integration boundary. The
[agent README](README.md) gives the rest of the agent.

## Runner handoff

The source-built runner hook reads its original GitHub assignment before
any action setup. The guest measures the actual Docker bridge proxy URL and
the preloaded Docker bytes before the runner can take the job.
It sends the original runtime credentials and permissions over vsock.
The hosted control plane receives only the runner and job identities.
Trusted facts from the hosted `JobPreparation` message then permit
machine-cache registration.
The machine verifies the runtime token outside the VM.
No administration credential enters a VM.

Action setup waits for the agent's decision, with no time limit in the
runner. GitHub's job cancellation and time limit stay the backstop.
The agent saves its decision before it sends it: `MachineCache` with the
sealed original handoff, or `GitHubOnly` for the whole job.
The agent seals a first handoff only when it arrives on the connection that
carried the fresh runner configuration. A boot connection that failed before
configuration does not use up that chance. A lost configuration connection
does.
A first handoff after a reconnect, a configuration retry, or an agent
restart gets `GitHubOnly`. That job uses only GitHub's cache, so it misses
the entries that only the machine cache keeps.
When a received job cannot get its facts after a control-plane loss, a
drain, a removal, or a stop request, the agent releases it with
`GitHubOnly`. A stop request does not stop that received job.
The hosted side also sends `GitHubOnly` when its App snapshot lacks the
job's repository. The agent applies it only to the matching running job
that has no facts and whose preparation still waits. It saves `GitHubOnly`
before it releases the Worker, and that decision is final. A repeated
delivery sends the same decision again.
For a job that already has facts, or whose preparation started or ended,
that message changes nothing. Later facts cannot restore the machine cache.

The seal keeps the rights and the URL origins of the first handoff, with a
host-decision marker. An origin is the scheme, host, and port of a URL.
The token, the URL paths, and the queries stay in memory.
A seal without that marker fails to decode, so no older seal is trusted.
A later replay must match both saved origins and cannot add an absent
endpoint. The saved rights replace the rights of the replay.
A sealed replay can recover its cache session, also from saved facts while
the control plane is unavailable.
A mismatch, or a running VM without a seal, uses GitHub's cache.

## Machine cache service

Install obtains the checked `ciproxy-cache-sidecar` release and creates
`ciproxy-cache.service`. The unit requires the XFS area mount.
Its stop order closes cache handles before unmount.
The service runs as the dedicated `ciproxy-cache` system user.
The root-owned `/var/lib/ciproxy/area/cache` directory stays private.
Its `data` child holds the cache files and belongs to that user.
The unit binds only that child at `/run/ciproxy-cache/data` for the service.
Before the service starts, its root helper copies the administration
credential into `/run/ciproxy-cache-auth/admin-token`.
That file has root ownership, the cache group, and mode `0640`.
Its root-owned parent has mode `0710` and permits cache-group traversal.
The service can read the copy. It cannot write or replace it.
The unit removes the copy after it stops, including a failed start.
Ordinary firewall repair does not copy the credential.
The original credential and the agent state stay private to root.
The service has no kernel capabilities and cannot gain privileges.
The unit omits `--capacity-bytes`. A restarted service therefore keeps its
entries and writes no new archive until the agent sets its budget.
Its administration endpoint is `127.0.0.1:8229`.
Its private credential is `/var/lib/ciproxy/cache-admin-token`.
VMs can reach only the job listener on port 7887.
The host firewall excludes other inbound interfaces from that listener.

The output firewall applies only to the cache user's sockets.
It blocks host-local addresses, non-public IPv4 ranges, and IPv6.
This includes metadata addresses and the host's public IPv4 addresses.
Only DNS to the current `/etc/resolv.conf` resolvers bypasses those blocks.
Both UDP and TCP DNS work, including an IPv6 resolver.
Public upstream HTTPS stays available.
The service can reply to established VM cache and host administration
connections. That exception applies only in the reply direction.
An established connection that the cache started cannot bypass the blocks.

The private files `/etc/ciproxy-cache.nft` and
`/etc/ciproxy-cache-firewall` hold the policy and its loader.
The unit runs the loader as root before each listener start.
This includes boot and each automatic restart.
The loader reads the current resolvers and applies the rules in one nftables
transaction. A failed load keeps the listener stopped.
Install and hourly repair create the system user and restore these files.
They apply the policy before they download a release.
A changed existing unit stops before it moves cache data into the child.
It tries to start under the new user before the release download.
If that protected start fails, checked release repair continues.
A failed download cannot leave that old service running as root.
Uninstall stops the service before it removes both private files and rules.
It also removes an interrupted start's transient credential.
The inactive system account remains to avoid reusing its UID.

## Cache budgets and recovery

Before a VM starts or a Docker disk takes space, the cache acknowledges its
smaller byte budget.
The cache waits for active file reads and writes before that acknowledgement.
If administration fails, the agent stops the service and removes its
occupied cache files. The job then uses GitHub or sees a cache miss.
Offline recovery reclaims both the parent cache directory and its `data`
child. This also releases bytes after an interrupted data move.
If the host cannot close handles or release files, admission waits for repair.
A Docker disk is then not attached, and its job uses the root disk.
A missing service alone does not block admission.
The periodic budget refresh allocates no space. When it fails, the agent
starts the service if necessary and tries again later.
This failure recovery does not run the offline cache clear.
Job exit revokes sessions by the persisted trusted assignment.
This also applies after restart.
The assignment includes organization, repository, run, job, and runner identity.
Reconnect recovers that assignment's live credential before registering again.
When the service lost that session, a replay with the saved origins
registers again with the saved rights.
Runtime credentials remain in memory.
The persisted setup phase prevents another Docker attachment after setup starts.
A failed revoke stops the service to invalidate its sessions.
ClearCache still requires confirmed deletion.

## Docker disks

Checked-scope Docker ownership, copies, drive switching, and guest setup
share a bounded attachment path.
Each active Docker lease persists its repository, writer access, and
reserved logical bytes. The lease is the only Docker ownership record.
A repository has at most one writer lease at a time.
The agent counts the leased bytes before it admits a new VM root.
Cleanup removes the job's Docker disk before its lease, so a failed cleanup
keeps the reservation. A file that is already gone counts as removed.
Each lease also records drive handoff and failed attachment recovery.
A failed mount first waits for Firecracker to close the drive.
Recovery discards a failed writer's base and the failed job attachment.
It keeps the reserved bytes until that job exits.
A failed copy cannot discard a source held by a running writer.
Offline checks discard an unusable idle source before another job uses it.
A failed Firecracker switch preserves the source because it proves no
filesystem failure. Reconnect and periodic reconciliation retry pending
drive closure. Repeated guest outcomes cannot discard a replacement base.
Disposable Docker disks use the same reflink copy as VM roots.
Only hosted spare disk can grant a new lease. A no-fit disk uses root Docker.
Filesystem growth checks actual free blocks before it acknowledges readiness.
The hosted spare adapter uses each machine's selected image.
Live KVM proof remains required.

## Related docs

- [Cache](../../docs/protocol/cache.md) defines the cache rules and the
  cache handoff.
- [Security](../../docs/protocol/security.md) defines job isolation.
- [Smoke checks](smoke-checks.md) give the live cache and Docker checks.
