# Runner cache patch

This Apache-2.0 patch sends the cache calls of an own-machine job through
the [cache sidecar](../../crates/ciproxy-cache-sidecar/README.md). It keeps
the runner's managed string table and PE offsets in place. Boat runs the
original runner and does not use this directory.

## Patch a runner

Run from the repository root:

```sh
python3 image/runner-patch/patch-runner.py \
  /path/to/actions-runner/bin/Runner.Worker.dll
python3 image/runner-patch/patch-runner.py \
  /path/to/actions-runner/bin/Runner.Worker.dll --check
```

Use `--output PATH` to write a separate patched DLL. The script replaces
the first letter of these strings with `Z`:

- `ACTIONS_CACHE_URL`
- `ACTIONS_CACHE_SERVICE_V2`
- `ACTIONS_RESULTS_URL`

It supports the UTF-16LE managed string table and ASCII. It preserves byte
length. A repeated patch keeps the same bytes. Missing strings or a mixed
patch stop before the destination changes. The script does not change
runtime tokens, artifact endpoints, or log endpoints.

Runner v2.338.2 is the verified baseline. Its official Linux x64 archive
has SHA256
`baebf204307bac6193480852732f0aba2ae97b6c5597c244f4d5558390147c40`.
The original DLL has 639488 bytes. It has one UTF-16LE instance of each
target string. The patch changes three bytes in that DLL.

## Start the job VM runner

The VM knows its proxy address before the runner listener starts. Start
the Docker daemon inside the VM first. Derive a VM address that Docker
actions and job containers can reach. Then start the runner listener:

```sh
cache_proxy_url="$(image/runner-patch/container-proxy-url.sh)"
image/runner-patch/run-with-cache.sh "$cache_proxy_url" ./run.sh
```

GitHub assigns a job after the listener starts. At job start, the existing
agent hook must get trusted facts and register that assignment. It must
start the VM proxy and wait for readiness before it returns. The
[sidecar guide](../../crates/ciproxy-cache-sidecar/README.md#agent-integration)
gives the handoff contract. A normal shell hook does not receive original
cache runtime credentials. Agent and image integration must supply them
through a trusted assignment handoff before action steps run.

The wrapper preserves the existing job-start hook. Its chained hook keeps
that hook's supported shell and exit status. A `.sh` file needs no
executable bit. The chain appends the three proxy values to `GITHUB_ENV`.
The runner then gives them to Docker actions and job-container steps.
The wrapper also sets them in the listener's parent environment.

The patched runner writes GitHub's original values to `ZCTIONS_*` instead.
Its `ACTIONS_RUNTIME_TOKEN` remains the original runtime token. Cache v1
and v2 requests reach the proxy. Other results requests go to GitHub.
The proxy can use GitHub directly when machine metadata is unavailable.
A lost transfer ticket produces the normal unavailable-cache result.

`container-proxy-url.sh` prints the gateway of Docker's VM-local `bridge`
network. Set `CIPROXY_DOCKER_BRIDGE` to use another existing bridge. A
typical result is `http://172.17.0.1:8230`. Set the registered `proxy_url`
to that exact value. Bind the proxy to `0.0.0.0:8230` inside the job VM.
The VM's network policy must keep other VMs out of that listener. A
loopback URL works for host processes only. It cannot serve Docker jobs.

## Local executable smoke

The fixture uses only Python's standard library. It needs no GitHub,
Boat, or KVM credential. Build first:

```sh
cargo build --package ciproxy-cache-sidecar
python3 image/runner-patch/smoke/check.py
```

The check starts the real machine and proxy binaries. It uploads a random
50 MiB archive in reversed block order. It retries a block and commits the
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
It starts both local executables and runs the official SDK with a unique
key. It checks save, lookup-only, and a content round trip. Expected
result: the SDK `PASS` line and an authenticated GitHub `PASS` line.
It removes the temporary proxy file with the runtime token on exit.
GitHub keeps the unique smoke cache entry until its normal eviction.

This check proves authenticated HTTP compatibility. It does not start a
Firecracker VM or a Boat sandbox. Agent and image integration own packaging,
runner startup, VM network isolation, and lifecycle smoke tests.

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
