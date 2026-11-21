# ciproxy-cache-sidecar

A transparent GitHub Actions cache for customer machines. The machine
service owns persistent entries and measured reports. A proxy in each job
VM keeps the runner interface unchanged. The library depends only on
`ciproxy-protocol` inside the workspace. Its tests also use the Apache-2.0
`ciproxy-agent` library. It has the Apache-2.0 licence in
[LICENSE](LICENSE).

## Responsibilities

Serve GitHub cache v1 and v2 requests. Keep repositories and pull requests
apart. Keep hosted code out of the public dependency tree. Let cache faults
fall back to GitHub or produce an unavailable cache result.

## What This Crate Does

A read selects one entry from the local entries and the trusted GitHub
entry metadata. Lookup searches each allowed ref in order: own ref, pull
request base ref, default ref. It checks an exact key before prefix
matches. A prefix match uses the newest entry. For a checked ref, this
order covers both stores, as
[Restore order](../../docs/protocol/cache.md#restore-order) requires.
A selected GitHub entry streams through the machine, which can keep a local
fill. Saves stream to GitHub and capture a local copy. Only accepted saves
own immutable local keys for a repository, ref, key, and version. The
machine keeps them after GitHub deletes or evicts its copy.

The machine checks each original runtime token at admin registration.
It verifies RS256, GitHub's issuer, the validity interval, and the expected
repository, run, runner, and
[job ID pair](#runtime-token-job-identity) from `JobFacts`.
Exactly one signed scope with the write bit supplies the full ref.
The machine keeps raw scope keys, including GitHub's tag scope keys.
It ignores job metadata for scope, repository, permission, destination, and
counts. A GitHub fill stays visible only within the reader's own scope.
The service keeps fills separate from accepted saves. A fill cannot prevent
a later save in that scope. An accepted save takes precedence over a fill
of the same scoped key. Existing records without provenance stay readable
as fills.

Each restore asks for current trusted entry metadata. The service queues
that request on its private admin listener, and the root agent answers it
from the hosted GitHub App listing. A restore waits up to 95 seconds for
that answer. Jobs never supply this metadata. A local entry keeps its
trusted source ref and creation time apart from its visibility scope. The
resolver ranks candidates by the source facts. Only the visibility scope
decides which sessions can read an entry. A restart and a GitHub deletion
or eviction keep both. An accepted save keeps its GitHub entry ID. It takes
its source facts from the first listing that shows that entry. A fill gets
its source facts only from the listing, never from the reader's ref. Exactly
one listed entry must match GitHub's result before and after the lookup.
Missing or inconsistent metadata gives a checked session GitHub's lookup
result or a miss. The service then keeps no fill with a guessed source.
An unknown session keeps its default-branch local reads. Without the root
relay, as in a standalone service, lookup uses local entries first, then
GitHub.

Only the trusted events in
[Trusted write-back](../../docs/protocol/cache.md#trusted-write-back) can
write shared machine-cache entries. Other events keep GitHub fills private
to the session or its pull request. A checked pull request writes only in a
`refs/pull/N/merge` ref. A checked unsafe default ref or pull request with a
normal branch ref cannot save. These saves return an unavailable-cache
result. A foreign head repository cannot write the default scope.
Direct GitHub writes that a workflow enables with `cache-mode: write`
follow GitHub's permission rules.

A missing, invalid, read-only, or ambiguous token gives `JobScope::Unknown`.
That session reads only default-branch entries locally. Its other restores
use GitHub. Permitted saves go to GitHub and a fresh private local scope.
No session restores that private scope, including its creator. Unknown
tokens never fail the job. Reports count unknown jobs by reason for seven
days. Each assignment keeps its first unknown reason on replay.

The machine refreshes public keys every five minutes. An unknown key ID
permits one extra refresh per minute across all jobs. Failures consume the
same cooldown. A failed periodic refresh expires the old key set. A failed
extra refresh can keep a still-current key set for known keys.

Azure single uploads, staged blocks, committed blocklist order, ranged
downloads, and legacy ranged PATCH uploads work. Lookup-only and HEAD
requests do not increase restore counts. A transfer counts once when its
served ranges cover the complete archive. Byte counts come from archive
data at the machine boundary. A failed machine connection returns a cache
miss. It starts no direct restore that could escape those counts. A known
machine-side direct fallback hides per-job counts. A service restart also
hides counts for jobs that were still active. Seven-day hit counts survive
a restart.

The v2 JSON interface accepts protobuf field names and camelCase names.
For example, both `size_bytes` and `sizeBytes` work. Int64 values remain
JSON strings, as in the official client.

Persistent archives and staged bytes share one disk budget. The service
evicts the least recently used entries. A smaller budget cancels staged
captures. Clear cancels pending publication and removes all local entry
files. It keeps report history. Neither operation calls GitHub deletion
or touches sibling Docker disks. Archive transfers use bounded chunks.
Each upload has one active archive: whole contents, committed ordered
blocks, or legacy ranges. A whole upload releases the earlier archive and
staged blocks before capture. Staged block writes keep committed bytes
unchanged until a successful block-list commit. That commit selects the
new archive and releases unselected staged bytes. A missing capture leaves
no usable copy of that version. Assembly never selects an older archive
form. GitHub still receives the upload when local capture fails.

Other results requests go directly from the VM proxy to GitHub. The
machine listener also supports results passthrough. Artifacts and logs
therefore remain independent of cache availability. An opaque transfer
ticket can become unavailable during an outage. The cache action then
reports its usual cache warning or miss.

### Runtime token job identity

`JobFacts.assignment.job_ids` is the `JobIdBinding` of the exact registered
runner: its Worker job ID, which the runner's Worker gets with the job, and
its scale-set job ID. The signed top-level `job_id` claim must equal one of
the two. Another value or a missing claim gives `JobMismatch`. No job name
or alias matches, and every other token check above also applies.
[Trusted write-back](../../docs/protocol/cache.md#trusted-write-back) gives
the rule and the rejected single-ID check.

Live run 44202592271 of `Firna-AI/ciproxy-spike` used control plane
`9b094e15` and agent 0.1.2. Its runtime token verified as RS256 against
[GitHub's key set][jwks] with key ID `38826b17-6a30-5f9b-b169-8beb8202f723`.
Its signed `job_id`, scale-set job ID, and Worker job ID were all
`6d88988c-67b0-5e52-87a7-a2fa0c927e5e`. So the claim named the bound job.
The run cannot show which ID the claim holds when the two IDs differ.

The [job identity sources](job-identity-sources.md) trace the Worker job ID
through pinned GitHub sources and give their limit.

[jwks]: https://token.actions.githubusercontent.com/.well-known/jwks

## Quick Start

Run the executable smoke check from the repository root:

```sh
cargo build --package ciproxy-cache-sidecar
python3 image/runner-patch/smoke/check.py
```

Start a machine service with a dedicated cache directory:

```sh
cargo run --package ciproxy-cache-sidecar -- machine \
  --listen 127.0.0.1:8228 --admin-listen 127.0.0.1:8229 \
  --cache-dir .context/cache --capacity-bytes 1073741824 \
  --admin-token-file .context/cache-admin-token
```

The agent creates the token file before startup. It contains an independent
random secret. The service has no anonymous administration route.

`--capacity-bytes` is optional. Without it, the budget is unknown. The
service then recovers its retained entries, writes no new archive, and
evicts nothing until `PUT /budget` sets a budget. With an explicit
capacity, also zero, startup reclaims the entries that do not fit it.
The agent's unit omits the option.
`GET /report` returns HTTP 503 until the budget is known and disk recovery
has succeeded. A later budget does not hide failed recovery.
A failed read or decode of saved counts also keeps reports unavailable.
Cache requests continue, and events do not overwrite those unread counts.

### Agent integration

The agent owns VM network, image, and lifecycle wiring. Use one
machine-service process for each dedicated cache directory. Expose its job
listener only through the narrow machine-cache connection. Keep its admin
listener and token outside every job VM. The default admin address is
loopback. The agent must not bind it to a VM-reachable interface.

The agent calls the admin listener with `Authorization: Bearer TOKEN`:

| Method and path            | Input or result                                             |
| -------------------------- | ----------------------------------------------------------- |
| `POST /sessions`           | `CacheRegistration` JSON. Returns `SessionCredential` JSON. |
| `DELETE /sessions/TOKEN`   | Revoke that VM's credential and finish its report.          |
| `POST /sessions/recover`   | Recover a live session with its exact registration.         |
| `POST /sessions/close`     | Revoke sessions with the exact trusted `JobFacts`.          |
| `POST /sessions/fallback`  | Hide counts for the exact trusted `JobFacts`.               |
| `GET /report`              | `ciproxy_protocol::cache::MachineCacheReport` JSON.         |
| `POST /clear`              | Clear this machine's local entries. Returns HTTP 204.       |
| `PUT /budget`              | `{"capacity_bytes":12345}`. Returns HTTP 204.               |
| `PUT /metadata-relay`      | `{"enabled":true}`. Returns HTTP 204.                       |
| `GET /metadata-requests`   | Waits briefly. Returns one `CacheMetadataWork` or `null`.   |
| `POST /metadata-responses` | One `CacheMetadataAnswer`. Returns HTTP 204.                |

The metadata routes serve the root agent's relay. The relay starts off in
each service process. Turn it on before you register or recover sessions.

`CacheRegistration` contains `job_id`, `facts`, `permissions`, `github_token`,
`cache_url`, `results_url`, and `proxy_url`. The exact types live in
[protocol runtime types](../ciproxy-protocol/src/runtime.rs).
`job_id` is the Worker job ID of the original handoff.
`facts` is the existing protocol `JobFacts`, with no full job ref. The
hosted control plane fills it from API facts, the scale-set run ID, the JIT
runner ID, and the `JobIdBinding` of that exact registered runner.
Close and fallback select the sessions with the exact `facts` and the
Worker job ID of that binding.
The agent supplies
original runtime credentials and endpoint URLs from the assigned runner
before job code runs. A job request cannot register or replace them.

The admin result contains `token` and `scope`. Keep the opaque session token
for the VM proxy. Use the typed `scope` for the agent's Docker-disk policy.
Only a checked exact default-branch ref of a trusted event can persist a
Docker disk, with the same foreign-head defense. Every unknown job gets a
disposable copy. The VM and job listener cannot issue admin registrations.

`permissions` has independent `read` and `write` booleans. Both default to
false. The agent must get the effective cache mode from trusted runner
assignment data. It must not read a job's environment override. Missing
permission information leaves the cache unavailable. The machine and proxy
both apply these permissions.

The own-machine guest receives `CacheProxyConfig` through its parent pipe.
It contains `machine_url`, `machine_token`, and the same `registration`.
The original runtime token never needs a configuration file.
`proxy_url` uses the actual Docker bridge gateway.
The trusted source hook pauses before action setup.
The guest acknowledges readiness after its proxy has bound the listener.
A failed machine cache returns a miss or leaves the GitHub endpoints.

The file input stays available for isolated operator checks:

```sh
ciproxy-cache-sidecar proxy --listen 0.0.0.0:8230 \
  --config /run/ciproxy/cache-proxy.json
```

The guest uses `--config-stdin --ready-stdout` instead.
Its pipe starts with one newline-delimited `CacheProxyConfig`.
The proxy acknowledges its bound listener with `READY`.
Later lines carry `CacheProxyUpdate` with a sequence and `CacheConnection`.
The proxy acknowledges each update with `UPDATED SEQUENCE`.
Updates keep the original assignment and results credentials unchanged.
The guest keeps this proxy when the machine service restarts.
It retries failed credential updates without repeating Docker setup.
Diagnostics go to stderr. Credentials stay in memory.
The machine's budget acknowledgement waits for active file reads and writes.
Shrinking cancels captures before it releases occupied files.

The proxy supplies `X-Ciproxy-Session` to the machine. Each random transfer
ticket belongs to that session. Job requests cannot reuse another VM's
ticket. The credential grants no admin access. A session can suppress its
own counts through `POST /ciproxy/fallback`. That operation accepts no
counts. Machine-connection failures return a miss without a notification.
The agent must revoke the session when the job ends.

The trusted source hook stays inactive on Boat. Boat uses GitHub directly.
The agent owns host service installation and VM lifecycle wiring.

Production uses the fixed GitHub Actions JWKS endpoint. An isolated smoke
machine can use `--verification-config PATH`, or the operator environment
`CIPROXY_CACHE_VERIFICATION_CONFIG`. The JSON file has `jwks_url`.
The operator owns that file and environment outside all job VMs. No job,
request, or registration selects the key source. The issuer stays fixed.

## Development

The tests use real TCP listeners, files, and a Python fake GitHub service.
They need Python 3 and no credential. Run from the repository root:

```sh
cargo test --package ciproxy-cache-sidecar
cargo build --package ciproxy-cache-sidecar
cargo fmt --all -- --check
cargo clippy --package ciproxy-cache-sidecar --all-targets -- -D warnings
```

The [runner patch guide](../../image/runner-patch/README.md) gives the
executable smoke, SDK conformance, and authenticated GitHub commands.

### Key Code

- [service.rs](src/service.rs) defines the HTTP service and separate routers.
- [machine.rs](src/machine.rs) authenticates and routes machine requests.
- [proxy.rs](src/proxy.rs) owns VM forwarding and cache misses during outages.
- [storage/](src/storage/mod.rs) owns capture, persistence, and disk budgets.
- [resolver.rs](src/resolver.rs) applies one restore order to local and
  GitHub candidates.
- [candidates.rs](src/candidates.rs) gets current entry facts and selects one
  candidate. [restore.rs](src/restore.rs) serves it to v1 and v2 lookups.
- [metadata_relay.rs](src/metadata_relay.rs) queues entry-fact requests for
  the root agent.
- [session.rs](src/session.rs) defines the agent registration contract.
- [runtime_token/](src/runtime_token/mod.rs) verifies scopes with injected
  public-key and clock boundaries.
- [cache_tests.rs](tests/cache_tests.rs) starts the public HTTP tests.

### Related Docs

- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Cache protocol](../../docs/protocol/cache.md) defines cache requirements.
- [Job isolation](../../docs/protocol/security.md#job-isolation) defines VM
  network access.
