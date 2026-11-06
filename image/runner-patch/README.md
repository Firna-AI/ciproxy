# Runner cache patch

This Apache-2.0 patch sends the cache calls of an own-machine job through
the [cache sidecar](../../crates/ciproxy-cache-sidecar/README.md). It keeps
the runner's managed string table and PE offsets in place. Boat runs the
original runner and does not use this directory.

## Build the trusted assignment hook

The image uses the pinned official runner archive.
It replaces only `Runner.Worker.dll` with a source-built assembly.
`image/build-runner.sh WORK_DIR` downloads the pinned source, verifies its
checksum, applies `prepare-source.py`, and publishes the Worker.
The source requires the .NET SDK version in its `global.json`.
The assembly keeps the official identity and dependency references.
The image retains separate stock and own-machine Worker assemblies.
The trusted entry mode selects the own-machine assembly for vsock mode.
Boat mode restores the byte-identical official stock assembly before launch.
Each fresh VM or sandbox has one listener, so this selection has no race.
The image runs the original listener and other official assemblies.

`CiproxyAssignment.cs` reads `SystemVssConnection` from the original GitHub
job request before action setup. It reads the runtime token, cache URL,
results URL, job UUID, and effective `actions_cache_mode` there.
It does not read job environment values for those facts.
The four modes are `read`, `write`, `write-only`, and `none`.
Missing rights grant neither local read nor local write.

The hook connects to the supervisor's private Unix socket.
The guest passes the original values over vsock to its own agent.
The original runtime token never enters the hosted control plane.
The machine cache checks the signed token against hosted assignment facts.
The guest derives its proxy URL from the actual Docker bridge gateway.

The worker waits up to 30 seconds for preparation to begin.
A missing socket or unavailable preparation leaves the GitHub endpoints.
Before local setup, the guest sends `preparing` and waits for the worker's
acknowledgement. The worker then waits for final readiness or fallback.
This prevents a timeout from starting actions during a Docker mount change.
When the proxy is ready, the hook replaces the assignment's cache and
results URLs with that same container-reachable proxy address.
The original runtime token remains unchanged.
Proxy configuration goes through stdin and stays in memory.

The source hook stays inactive in Boat mode.
Boat uses the original GitHub cache endpoints.
No workflow hook or workflow edit is needed for ordinary cache use.

The byte-string patch and shell wrapper remain available for the isolated
M11 compatibility smoke below. Image provisioning uses the source hook.
They are not a source of trusted credentials or cache permissions.

## Local executable smoke

The fixture uses Python's standard library and OpenSSL. It needs no GitHub,
Boat, or KVM credential. Build first:

```sh
cargo build --package ciproxy-cache-sidecar
python3 image/runner-patch/smoke/check.py
```

The check starts the real machine and proxy binaries with a fixture RSA
token and a loopback JWKS source. The operator-owned verification file
selects those public keys for this smoke.
The check asserts the machine's checked default scope before cache calls.
It uploads a random 50 MiB archive in reversed block order.
It retries a block and commits the
intended order. It then deletes the fake GitHub copy and checks the local
restore, a ranged read, measured counts, results passthrough, and clear.
Expected result: one `PASS` line and the listener details. Logs and
configuration files stay under `.context/ciproxy-mvp-m11/`.

To keep the services available for a one-time SDK check:

```sh
python3 image/runner-patch/smoke/check.py --serve
```

Use the printed `proxy` and `directory` values in another terminal. Pass
the path of an installed official `@actions/cache` module. Do not add the
SDK as a product dependency just for this check:

```sh
export CIPROXY_SMOKE_PROXY=http://127.0.0.1:PROXY_PORT
export CIPROXY_SMOKE_DIR=/absolute/path/to/.context/sdk-check
node image/runner-patch/smoke/sdk-check.mjs \
  /absolute/path/to/node_modules/@actions/cache
```

The SDK check saves a random 50 MiB file, checks lookup-only behavior, and
restores by a restore key. It verifies the restored digest. Set
`CIPROXY_SMOKE_BYTES=135266304` to exceed the SDK's single-upload threshold
and exercise its staged Azure uploader. Tar and gzip must be installed.
Zstd is optional. This script is verification evidence, not a new gate.

The fake GitHub service can also run alone:

```sh
python3 image/runner-patch/smoke/fake-github.py --port 0
```

Its first stdout line gives its origin. The fixture has cache v1/v2,
signed blob endpoints, and `/_fixture/state`, `/_fixture/options`,
`/_fixture/seed`, and `/_fixture/entries` controls. Use it only for checks.

## Authenticated GitHub smoke

Run inside an assigned GitHub Actions runner with its original
`ZCTIONS_CACHE_URL`, `ZCTIONS_RESULTS_URL`, and `ACTIONS_RUNTIME_TOKEN`.
An unpatched runner can supply the original `ACTIONS_*` URLs instead.
Use a default-branch job with trusted read and write permission. Supply
the agent's trusted `Registration` JSON for that same job. Do not build
the facts from a job's environment. Install the official cache SDK in a
scratch directory first. Then run:

```sh
python3 image/runner-patch/smoke/live-check.py \
  --registration /run/ciproxy/cache-registration.json \
  --cache-module /absolute/path/to/node_modules/@actions/cache
```

The script uses the original runtime credentials for that assignment.
The trusted runner handoff or a parent Node action must supply that
environment. Do not print the token or put it on a command argument.
The script checks the host's returned scope against the assigned default
branch before the SDK writes anything.
It starts both local executables and runs the official SDK with a unique
key. It checks save, lookup-only, and a content round trip. Expected
result: the SDK `PASS` line and an authenticated GitHub `PASS` line.
It removes the temporary proxy file with the runtime token on exit.
GitHub keeps the unique smoke cache entry until its normal eviction.

This check proves authenticated HTTP compatibility. It does not start a
Firecracker VM or a Boat sandbox. Agent and image integration own packaging,
runner startup, VM network isolation, and lifecycle smoke tests.

Run the
[runtime-token release probe](../../crates/ciproxy-github/scripts/README.md#runtime-token-release-probe)
before each image release. It checks the current token format without
printing credentials or signature bytes.

## Upstream contracts

- [Runner action environment](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Worker/Handlers/NodeScriptActionHandler.cs)
  supplies the original runtime values.
- [Runner job hook](https://github.com/actions/runner/blob/v2.338.2/src/Runner.Worker/JobHookProvider.cs)
  processes `GITHUB_ENV` before action steps.
- [Cache v1 client](https://github.com/actions/toolkit/blob/main/packages/cache/src/internal/cacheHttpClient.ts)
  defines reservation, ranged PATCH, and commit calls.
- [Cache v2 schema](https://github.com/actions/toolkit/blob/main/packages/cache/src/generated/results/api/v1/cache.ts)
  defines protobuf JSON and string-encoded int64 fields. The sidecar accepts
  both `size_bytes` and `sizeBytes`, and both restore-key spellings.
- [Generated cache JSON client](https://github.com/actions/toolkit/blob/main/packages/cache/src/generated/results/api/v1/cache.twirp-client.ts)
  sends protobuf field names such as `restore_keys` and `size_bytes`.
- [Cache upload client](https://github.com/actions/toolkit/blob/main/packages/cache/src/internal/uploadUtils.ts)
  uses Azure single uploads and staged blocks.
- [GitHub cache access rules](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching)
  define branch lookup and low-trust event permissions.
