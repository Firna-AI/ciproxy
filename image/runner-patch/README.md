# Runner cache patch

This Apache-2.0 patch sends the cache calls of an own-machine job through
the [cache sidecar](../../crates/ciproxy-cache-sidecar/README.md). The image
installs a Worker built from the pinned runner source. Boat runs the
original runner and does not use the source hook.

## Build the trusted assignment hook

The image uses the pinned official runner archive.
It adds a source-built `Runner.Worker.dll` and raises two package
assemblies, as [Worker overlay](#worker-overlay) says.
`image/build-runner.sh WORK_DIR` downloads the pinned source, verifies its
checksum, applies `prepare-source.py`, and publishes the Worker.
The source requires the .NET SDK version in its `global.json`.
`image/build-runner.sh WORK_DIR osx-arm64` builds the Worker for the
`osx-arm64` runner of the macOS image into `WORK_DIR/patched-worker-osx-arm64`.
The runner source takes its OS constants, such as `OS_OSX`, from the build
host and not from `PackageRuntime`. So each Worker builds on its own OS, and
the script stops on another OS. The hook itself has no OS-specific code.
On macOS, the guest passes the socket `/var/run/ciproxy/assignment.sock`,
and the `cache_ready` decision names the loopback proxy
`http://127.0.0.1:8230`, because macOS jobs have no Docker.
The assembly keeps the official identity and dependency references.
The image retains separate stock and own-machine Worker assemblies.
The trusted entry mode selects the own-machine assembly for vsock mode.
Boat mode restores the byte-identical official stock assembly before launch.
Each fresh VM or sandbox has one listener, so this selection has no race.
The image runs the original listener and the other official assemblies,
except the packages that the Worker overlay raises.
`prepare-source.py` also adds the format probe
`Runner.Worker ciproxy-handoff-protocol-2`. The entry script runs it before
the listener starts, so the listener never starts with an older Worker.
The agent accepts only guests of handoff format 2, so own machines need an
image built from this source.
It also raises `Microsoft.AspNetCore.WebUtilities` in the Worker project to
8.0.34, the first 8.0 version that fixes GHSA-x33c-w48q-qrgc. The pinned
`Microsoft.DevTunnels.Connections` brings 8.0.25, and the runner's NuGet
audit fails the build on it. The audit stays on.

## Worker overlay

The official runner brings `Microsoft.AspNetCore.WebUtilities` and
`Microsoft.Net.Http.Headers` 8.0.25 for its dev tunnels. GHSA-x33c-w48q-qrgc
affects that WebUtilities version. Both the stock and the own-machine
Worker load these assemblies from the runner's `bin` folder. So the images
also install the 8.0.34 assemblies of the patched build, the versions that
the patched build resolves.

`worker-overlay.py` runs on the build host after the Worker build. It
reads `bin/Runner.Worker.deps.json` from the pinned runner archive. It
writes the overlay folder:

- `Runner.Worker.dll`, the patched Worker;
- each package assembly that the patched build raises over the runner;
- `Runner.Worker.deps.json`, the runner's own deps file with only the
  raised package entries and the references to them changed;
- `install-files.txt`, `packages.txt`, and `stock-deps.sha256`, the inputs
  of the install script.

The overlay does not copy the deps file of the patched build. The official
runner is self-contained: its deps file lists the .NET runtime pack, and
its `Runner.Common`, `Runner.Sdk`, and `Sdk` entries have version 1.0.0.
The patched build is framework-dependent and has other project versions.
Its deps file would leave the runtime out of the runner.

The script stops when the patched build lowers a package, adds a package
that the runner lacks, or changes the files of a raised package. It stops
when the runtimes differ. It also stops when a package that
`prepare-source.py` pins is below its pinned version in the overlay deps
file. It then compares the file version of each raised assembly with the
deps file.

`install-worker.sh RUNNER_DIR OVERLAY_DIR` installs the overlay in the
image. It needs no Python, and it runs on the Bash 3.2 of macOS. It stops
when the runner's deps file is not the one that the overlay was made from.
It keeps the stock `Runner.Worker.dll` as `Runner.Worker.stock.dll` and
installs the patched Worker as `Runner.Worker.own.dll`. It copies the raised
assemblies and the deps file into `bin`. It then checks that each file is in
place and that the deps file names only the raised version of each raised
package. The Linux machine image, the Boat image, and the macOS image use
the same two scripts.

`CiproxyAssignment.cs` reads `SystemVssConnection` from the original GitHub
job request before action setup. It reads the runtime token, cache URL,
results URL, job UUID, and effective `actions_cache_mode` there.
It does not read job environment values for those facts.
The four modes are `read`, `write`, `write-only`, and `none`.
Missing rights grant neither local read nor local write.

The hook connects to the supervisor's private Unix socket.
Each message on it carries handoff format 2 and the job ID.
The guest passes the original values over vsock to its own agent.
The original runtime token never enters the hosted control plane.
The machine cache checks the signed token against hosted assignment facts.
The guest derives its proxy URL from the actual Docker bridge gateway.

The hook holds action setup until the host's matching decision arrives.
It has no time limit of its own.
A missing socket, a socket error, an end of stream, or an invalid reply
makes it connect again after one second. None of them starts the steps.
Before local setup, the guest sends `preparing` and waits for the worker's
acknowledgement.
The decision `github_only` keeps the original GitHub endpoints for the
whole job. The decision `cache_ready` replaces the assignment's cache and
results URLs with that same container-reachable proxy address.
GitHub's job cancellation still ends the wait. The Worker then returns
`Canceled` before any step starts.
The original runtime token remains unchanged.
Proxy configuration goes through stdin and stays in memory.

The source hook stays inactive in Boat mode.
Boat uses the original GitHub cache endpoints.
No workflow hook or workflow edit is needed for ordinary cache use.

## Host decision check

This opt-in check runs the actual patched Worker hook against local
host-decision fixtures. It adds no checked-in .NET project and no CI gate.
It needs the .NET SDK. Build the Worker first with
`image/build-runner.sh WORK_DIR`. Then pass that output and the official
runtime assemblies that the image keeps beside the patched Worker:

```sh
python3 image/runner-patch/tests/host-decision.py \
  --worker-output BUILD_DIR/patched-worker \
  --runner-bin OFFICIAL_RUNNER_DIR/bin \
  --dotnet dotnet
```

The cases are `early_eof`, `connect_error`, and `github_cancellation`.
`--case NAME` runs one case. The cancellation case also runs the actual
`JobRunner` and checks its `Canceled` result with no started step.
The check writes its project, build output, and sockets under `.context/`.
It has no retry or repeat option.

## Overlay load check

This opt-in check installs a Worker overlay into a fresh copy of the
official runner. It then starts the runner's own `Runner.Worker` host with
the stock and the own-machine Worker. A .NET startup hook,
`OverlayLoadHook.cs`, loads each raised package from `bin`, checks its
version, and runs code in it. The check then runs the format probe of the
own-machine Worker and `Runner.Listener --version`. It needs the .NET SDK
and runs on the OS of the runner archive:

```sh
python3 image/runner-patch/worker-overlay.py \
  --worker-output BUILD_DIR/patched-worker \
  --runner-archive RUNNER_ARCHIVE --output BUILD_DIR/worker-overlay
python3 image/runner-patch/tests/overlay-load.py \
  --runner-archive RUNNER_ARCHIVE --overlay BUILD_DIR/worker-overlay
```

Each step prints `PASS` or `FAIL`. The check writes the runner copy and
the hook build under `.context/overlay-load/`.

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

Run inside an assigned stock GitHub Actions runner with its original
`ACTIONS_CACHE_URL`, `ACTIONS_RESULTS_URL`, and `ACTIONS_RUNTIME_TOKEN`.
With the source hook active, steps see the proxy URL instead of the original
cache and results URLs.
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
A parent Node action must supply that environment.
Do not print the token or put it on a command argument.
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
