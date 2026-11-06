# Cache and Docker integration

The agent connects each job VM to the machine cache of its host and to a
checked Docker disk. This guide gives that integration boundary. The
[agent README](README.md) gives the rest of the agent.

## Runner handoff

The source-built runner hook reads its original GitHub assignment before
any action setup. The guest derives the actual Docker bridge proxy URL.
It sends the original runtime credentials and permissions over vsock.
The hosted control plane receives only the runner and job identities.
Authenticated JobFacts then permit machine-cache registration.
The machine verifies the runtime token outside the VM.
The guest acknowledges local proxy readiness before action setup resumes.
No administration credential enters a VM.

The agent saves the rights and the URL origins of the first handoff that it
accepts for a VM. An origin is the scheme, host, and port of a URL.
The token, the URL paths, and the queries stay in memory.
A later replay must match both saved origins and cannot add an absent
endpoint. The saved rights replace the rights of the replay.
A mismatch, or a running VM without saved origins, uses GitHub's cache.

## Machine cache service

Install obtains the checked `ciproxy-cache-sidecar` release and creates
`ciproxy-cache.service`. The unit requires the XFS area mount.
Its stop order closes cache handles before unmount.
The service uses `/var/lib/ciproxy/area/cache`.
The unit omits `--capacity-bytes`. A restarted service therefore keeps its
entries and writes no new archive until the agent sets its budget.
Its administration endpoint is `127.0.0.1:8229`.
Its private credential is `/var/lib/ciproxy/cache-admin-token`.
VMs can reach only the job listener on port 7887.
The host firewall excludes other inbound interfaces from that listener.

The unit loads that firewall rule set from the private file
`/etc/ciproxy-cache.nft` before each listener start. This includes boot and
each automatic restart. A failed load stops the start.
Install and the hourly repair write the file and the unit, and load the
rules, before they download a release. A failed download still leaves the
rules repaired. Uninstall stops the service before it removes the file.

## Cache budgets and recovery

Before a VM starts or a Docker disk takes space, the cache acknowledges its
smaller byte budget.
The cache waits for active file reads and writes before that acknowledgement.
If administration fails, the agent stops the service and removes its
occupied cache files. The job then uses GitHub or sees a cache miss.
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
