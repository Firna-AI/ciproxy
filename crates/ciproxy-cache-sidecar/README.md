# ciproxy-cache-sidecar

A transparent GitHub Actions cache for customer machines. The machine
service owns persistent entries and measured reports. A proxy in each job
VM keeps the runner interface unchanged. This crate depends only on
`ciproxy-protocol` inside the workspace. It has the Apache-2.0 licence in
[LICENSE](LICENSE).

## Responsibilities

Serve GitHub cache v1 and v2 requests. Keep repositories and pull requests
apart. Keep hosted code out of the public dependency tree. Let cache faults
fall back to GitHub or produce an unavailable cache result.

## What This Crate Does

Reads use the machine first, then GitHub with local fill. Saves stream to
GitHub and capture a local copy. Only accepted saves own immutable local
keys for a repository, ref, key, and version. The machine keeps them after
GitHub
deletes or evicts its copy. Lookup searches each allowed ref in order:
own ref, pull request base ref, default ref. It checks an exact key before
prefix matches. A prefix match uses the newest entry.

The machine checks each original runtime token at admin registration.
It verifies RS256, GitHub's issuer, the validity interval, and the expected
repository, run, runner, and optional job identity from `JobFacts`.
Exactly one signed scope with the write bit supplies the full ref.
The machine keeps raw scope keys, including GitHub's tag scope keys.
It ignores job metadata for scope, repository, permission, destination, and
counts. A GitHub fill stays within the reader's own scope because the
GitHub lookup response does not identify the source ref. The service keeps
fills separate from accepted saves. A fill cannot prevent a later save in
that scope. An accepted save takes precedence over a fill of the same
scoped key. Existing records without provenance stay readable as fills.

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

### Agent integration

The agent owns VM network, image, and lifecycle wiring. Use one
machine-service process for each dedicated cache directory. Expose its job
listener only through the narrow machine-cache connection. Keep its admin
listener and token outside every job VM. The default admin address is
loopback. The agent must not bind it to a VM-reachable interface.

The agent calls the admin listener with `Authorization: Bearer TOKEN`:

| Method and path           | Input or result                                             |
| ------------------------- | ----------------------------------------------------------- |
| `POST /sessions`          | `CacheRegistration` JSON. Returns `SessionCredential` JSON. |
| `DELETE /sessions/TOKEN`  | Revoke that VM's credential and finish its report.          |
| `POST /sessions/recover`  | Recover a live session with its exact registration.         |
| `POST /sessions/close`    | Revoke sessions with the exact trusted `JobFacts`.          |
| `POST /sessions/fallback` | Hide counts for the exact trusted `JobFacts`.               |
| `GET /report`             | `ciproxy_protocol::cache::MachineCacheReport` JSON.         |
| `POST /clear`             | Clear this machine's local entries. Returns HTTP 204.       |
| `PUT /budget`             | `{"capacity_bytes":12345}`. Returns HTTP 204.               |

`CacheRegistration` contains `job_id`, `facts`, `permissions`, `github_token`,
`cache_url`, `results_url`, and `proxy_url`. The exact types live in
[protocol runtime types](../ciproxy-protocol/src/runtime.rs).
`facts` is the existing protocol `JobFacts`.
The hosted `JobAssignments` builder combines API facts with the scale
message and JIT runner identity. `facts` has no full job ref.
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
- [session.rs](src/session.rs) defines the agent registration contract.
- [runtime_token/](src/runtime_token/mod.rs) verifies scopes with injected
  public-key and clock boundaries.
- [cache_tests.rs](tests/cache_tests.rs) starts the public HTTP tests.

### Related Docs

- [Architecture](../../docs/architecture.md) defines component boundaries.
- [Cache protocol](../../docs/protocol/cache.md) defines cache requirements.
- [Job isolation](../../docs/protocol/security.md#job-isolation) defines VM
  network access.
