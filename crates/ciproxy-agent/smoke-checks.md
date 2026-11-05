# KVM and GitHub smoke checks

Use a disposable private repository with the staging GitHub App installed.
Copy [the workflow fixture](scripts/github-smoke-workflow.yml) into that
repository as `.github/workflows/ciproxy-agent-smoke.yml`.
Push it to the branch that the scripts will dispatch.
The staging gateway must route its `ciproxy` jobs to this machine.
It must supply a verified image and keep a warm runner for the warm check.
The image must include the guest program and Docker.
Authenticate `gh` to dispatch and read the test workflow.
Run each check as root on the registered KVM machine:

```sh
export CIPROXY_SMOKE_REPO=owner/private-agent-smoke
export CIPROXY_SMOKE_REF=main
crates/ciproxy-agent/scripts/smoke-docker.sh
crates/ciproxy-agent/scripts/smoke-warm.sh
crates/ciproxy-agent/scripts/smoke-restart.sh
```

The [ext4 host smoke](host-preparation.md#ext4-and-kvm-smoke) also checks
first install, repair, two real reflink jobs, and uninstall.

Optional inputs are `CIPROXY_SMOKE_WORKFLOW` and `CIPROXY_SMOKE_STATE`.
The restart check restarts the agent during a real Docker job.
It checks the same VM process and a successful job result.
It also requires a fresh accepted JobStarted observation from the new agent.
The script filters structured JSON journal events by service invocation.
The observation contains only the generated local VM ID.
The warm check proves that the runner was ready before dispatch.
It snapshots every VM ID before dispatch, including Starting VMs.
The replacement warm VM must have an ID absent from that snapshot.
An active machine that never reaches warm readiness fails the check.
The smoke host also needs journalctl.
Each check waits for the used VM and disks to disappear.
Evidence stays under `.context/ciproxy-mvp/`.
Missing resources produce `SKIP` and exit 77. Failed checks exit 1.
