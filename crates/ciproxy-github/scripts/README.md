# Live GitHub probes

Install Python 3, OpenSSL, and `gh` on Linux.
Keep the App JSON outside tracked files. It has `id` and `pem` fields.
The `id` accepts a number or a string. The `pem` holds the private key.
Authenticate `gh` separately as an operator who can create fixture refs
and workflows, cancel their runs, and read the organization plan.
The App keeps its three permissions. Tokens never go on command arguments.

```sh
export CIPROXY_GITHUB_APP_PATH=/absolute/path/to/app.json
export PYTHONPYCACHEPREFIX="$PWD/.context/ciproxy-mvp/milestone-6/pycache"
python3 crates/ciproxy-github/scripts/runner_group_probe.py --help
python3 crates/ciproxy-github/scripts/ref_metadata_probe.py --help
python3 crates/ciproxy-github/scripts/push_ref_probe.py --help
python3 crates/ciproxy-github/scripts/run_job_ref_probe.py --help
python3 crates/ciproxy-github/scripts/workflow_ref_probe.py --help
```

Each help page gives its prerequisites, resource scope, and cleanup steps.
Use two private disposable repositories with initial commits for the
selected-group probe. Install the same App on both repositories.
Provide an executable launcher for a clean, unconfigured Linux runner.
It receives JIT only in `ACTIONS_RUNNER_INPUT_JITCONFIG`.
It copies the runner into `CIPROXY_GITHUB_PROBE_WORK_DIR` and runs it there.
The script withholds operator credentials from the runner environment.

```sh
python3 crates/ciproxy-github/scripts/runner_group_probe.py \
  --allowed-repo Firna-AI/ciproxy-spike \
  --refused-repo Firna-AI/ciproxy-spike-refused \
  --runner-launcher /absolute/path/to/launcher
python3 crates/ciproxy-github/scripts/ref_metadata_probe.py \
  --repo Firna-AI/ciproxy-spike --create-temp
python3 crates/ciproxy-github/scripts/push_ref_probe.py \
  --repo Firna-AI/ciproxy-spike
python3 crates/ciproxy-github/scripts/run_job_ref_probe.py \
  --run "pull-request=Firna-AI/ciproxy-spike:$PR_RUN_ID"
```

Set `PR_RUN_ID` to an existing run. Repeat `--run` for other supplied cases.
The push probe creates both same-name branch and tag push runs itself.
It needs no existing workflow or run ID. It uses `ubuntu-latest` and one
`true` step. It needs no ciproxy runner or Boat account.
The probes read no job code, logs, artifacts, or cache contents.
They print safe JSON records and remove their unique resources.
Check every cleanup result. `free_proven` requires successful execution
on a GitHub organization whose reported plan is `free`.
No live success is claimed by the local HTTP tests.

The runner-free workflow-ref probe uses the existing `facts.yml` and
`facts-target.yml` fixtures. It reserves the unused `ciproxy-facts` label
and recreates the approved temporary `facts-pr` head. It collects direct
and reusable assignments for branch and tag pushes, tag dispatch, PR, and
`pull_request_target`. It creates no runner or JIT configuration.
The output pairs four scale-message fields with trusted API run facts.
It selects no ref source or cache policy.

```sh
python3 crates/ciproxy-github/scripts/workflow_ref_probe.py \
  --repo Firna-AI/ciproxy-spike --expected-default-sha a76c008
```

The script preserves the default branch, cancels and deletes its runs,
closes its PR, and removes its temporary refs, session, scale set, and
runner group. The closed PR record stays on GitHub. Its help page gives
the exact fixture contract and expected job counts.

Run all commands from the repository root.
See the [crate guide](../README.md) for service boundaries and local tests.

Run the offline job-metadata regression without GitHub credentials:

```sh
python3 -B crates/ciproxy-github/scripts/test_probe_group_flow.py -v
```

## Runtime-token release probe

Run `runtime_token_probe.py` in the private test repository before each
image release. It needs Python 3 and OpenSSL. It needs the original
`ACTIONS_RUNTIME_TOKEN` of its current job. It uses no App key, operator
token, or extra GitHub permission. It checks RS256 against GitHub's public
JWKS, exact issuer, and validity before it prints anything from the claims.

Ordinary shell steps can lack the original runtime values. Invoke the
probe as a child of a Node action that already receives them. Make the
script available at an absolute path in that action. Its entry point can
use this code:

```javascript
const { spawnSync } = require('node:child_process');
const result = spawnSync('python3', ['-B', '/absolute/path/runtime_token_probe.py'], {
  env: process.env,
  stdio: 'inherit',
});
process.exit(result.status ?? 1);
```

The trusted runner handoff can instead supply the original environment:

```sh
python3 -B crates/ciproxy-github/scripts/runtime_token_probe.py
```

Keep the token in the inherited environment. Never echo it or put it on a
command argument. The probe prints only claim names, decoded `ac` scope
and permission pairs, and repository, run, job, and runner identity claims.
It prints no token, signature, other claim value, or exception body.
Failure returns a nonzero status and a fixed message with no token details.
Temporary OpenSSL inputs stay under `.context/` and are deleted on exit.

Check a default-branch push, a branch, a same-name tag, a pull request,
`pull_request_target`, a read-only cache job, and local and remote reusable
workflows. Check a ciproxy JIT job as well. Compare its repository and run
with the API and scale message. Compare its job UUID and runner ID with the
scale message and JIT registration. A read-only job has no write scope.
Keep raw scope keys such as `refs/heads/refs/tags/main` in the evidence.
Save only this safe output under `.context/` for the release review.
