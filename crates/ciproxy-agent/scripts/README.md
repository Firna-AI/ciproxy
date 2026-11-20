# Agent scripts

This Apache-2.0 folder holds release packaging, live operator checks, and
their local regression tests. A live check runs on a real x64 Linux host
and changes that host. The [agent README](../README.md#development) links
the guides of the other scripts.

## Release packaging

[cli-release.sh](cli-release.sh) packages the agent and cache sidecar for
the release workflow and local smoke checks. Run it from the repository
root. The [packaging guide](../development.md#release-packaging) gives the
build, glibc check, and archive commands.

## Job image wait

Each live smoke in this folder runs real GitHub jobs on the machine. Before
its first job, it waits until the machine has its job image. The
[smoke guide](../smoke-checks.md#wait-for-the-job-image) gives the rule,
the time limit, and the inputs of the wait.

## Cache firewall proof

[smoke-cache-egress.py](smoke-cache-egress.py) is the entry point of the
cache firewall proof. The proof checks the output firewall of the installed
machine cache, its protected starts, and a real reboot. The
[cache firewall proof guide](../cache-egress-smoke.md) gives the
preparation, the commands, the two phases, and the evidence.

The proof needs these things:

- Root and a usable KVM device.
- systemd, nftables, iproute2, `setpriv`, and Python 3.
- An installed agent and cache service from matching releases.
- Private input files at the paths in `CIPROXY_OWN_SMOKE_INPUT`,
  `CIPROXY_SMOKE_CREDENTIALS`, and `CIPROXY_CACHE_ADMIN_TOKEN_FILE`.
- A disposable registered host. The proof stops and starts the agent and
  the cache service. Set `"cache_firewall_disposable": true` in the private
  input JSON to confirm this.

When one of these is missing, the script prints `SKIP` and returns exit 77.
A skipped run is not a live proof. Wait for every existing job first.
A running job fails the proof.

[cache-firewall.sh](cache-firewall.sh) is not an operator script. The agent
embeds it and installs it as the firewall loader of the cache service.

### Readiness without a budget

The cache service starts with no byte budget. Only the running agent sends
the budget, also after a service restart. Until the budget arrives,
`GET /report` on the administration listener returns HTTP 503.

The proof restarts the cache service while the agent is stopped. So each
inspection of the cache checks readiness without a budget. It sends two
requests to `http://127.0.0.1:8229/`:

- With the token from `CIPROXY_CACHE_ADMIN_TOKEN_FILE`, the request must
  get HTTP 404.
- Without a token, the request must get HTTP 401.

These requests read no usage and change no cache state. The script never
sends a budget of its own. A guessed budget could evict cache entries.

### Usage report after the agent starts again

After the restarts, the script starts the agent again if the agent ran
before. The agent then sends its budget. Next, the script requires a
successful `GET /report` within 60 seconds. It retries only HTTP 503. Any
other error fails the proof. The script also requires the report after it
restores the cache service from the firewall-failure job.

## Local regression tests

These tests replace the host commands, files, clock, and HTTP listeners
with stand-ins. They need no root, KVM, private input, or network. Run them
from the repository root:

```sh
python3 -B -m unittest discover -s crates/ciproxy-agent/scripts/tests -v
```

They check these things:

- The network probes of the cache firewall proof.
- Readiness without a budget, and the usage report after the agent starts
  again.
- The readiness rule of the job image wait, also with an expected release.
- The time limit of the image wait, its default, and its override.
- The order of the smoke steps: the image wait comes before the first warm
  wait and the first dispatch, and after the repair of the ext4 host smoke.
- The cancel of the dispatched GitHub run when a shell smoke fails.

The image wait tests use a fake image index and a fake clock, so they never
sleep. The shell tests run the real shell smokes with stand-in commands.
They need `bash`, `jq`, and `rg`. The tests do not count as live proof. The
repository gate does not run them. Run them after you change these scripts.

## Key Code

- [cli-release.sh](cli-release.sh) checks release tags and glibc versions
  and writes the public binary archives with their licences and checksums.
- [cache_egress_host.py](cache_egress_host.py) inspects the installed
  cache, checks readiness, and waits for the usage report.
- [cache_egress_lifecycle.py](cache_egress_lifecycle.py) restarts the cache
  with the agent stopped and runs the firewall-failure job.
- [cache_egress_network.py](cache_egress_network.py) runs the network
  probes against the installed cache.
- [smoke_image.py](smoke_image.py) waits until the image index names a
  ready job image, for the shell and the Python smokes.
- [smoke-common.sh](smoke-common.sh) holds the shared steps of the shell
  smokes. `prerequisites` runs the image wait, and `fail` tries to cancel
  the dispatched GitHub run.
- [own_smoke.py](own_smoke.py) holds `Smoke`, the shared client of the
  Python smokes. `Smoke` runs the image wait when it starts.
- [tests/](tests/) holds the regression tests and their stand-in hosts.
