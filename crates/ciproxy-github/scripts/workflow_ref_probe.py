#!/usr/bin/env python3
"""Observe scale-set jobWorkflowRef for pushes, a tag dispatch, and PR events."""

import argparse
from pathlib import Path
import sys

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path.cwd() / "crates/ciproxy-github/scripts"))

import probe_common
from probe_common import App, ProbeError, entrypoint, repository
from wfref_flow import execute

PROCEDURE = """
Run from the repository root. This probe imports the shared probe helpers.

Environment:
  CIPROXY_GITHUB_APP_PATH  Required App JSON file with id and pem. id can be a string.
  GH_TOKEN / gh auth      Operator credentials for refs, workflow commits, PRs,
                         workflow dispatch, and cancellation/deletion of test runs.
  PYTHONPYCACHEPREFIX     Optional. Put caches under .context.

Prerequisites for the approved Firna-AI/ciproxy-spike fixtures:
  App grants only Self-hosted runners write, Metadata read, and Actions read.
  The ciproxy-facts scale-set name and runner label must be unused.
  facts-pr must not exist as a branch or an open PR. The owner approved recreating,
  opening, and closing this exact head. The script refuses an existing head.
  Default facts.yml has workflow_dispatch with no required inputs, and pull_request
  on the default branch. Its probe jobs route to ciproxy-facts and filter facts-pr.
  Default facts-target.yml has pull_request_target opened/synchronize/reopened,
  filters facts-pr, and routes its target job to ciproxy-facts.
  The default files and facts-reusable.yml already exist. No default edit is made.

Command for all five cases, without a runner or JIT:
  python3 WORKFLOW_REF_PROBE_SCRIPT \
    --repo Firna-AI/ciproxy-spike --expected-default-sha a76c008

The unique branch/tag push workflow has one 'true' step and no token permissions.
Tag dispatch uses a unique tag at the unchanged default SHA and the existing
facts.yml. The PR uses facts-pr and a harmless file. It triggers both PR fixtures,
also when only one PR case is selected. Capacity 8 covers their direct/reusable
jobs. Default expected jobs: dispatch 2, pull_request 3, pull_request_target 1.
Override these counts only when the operator's existing fixture has other jobs.

Only four job message fields are printed: jobWorkflowRef, eventName, jobDisplayName,
workflowRunId. Each is paired with allowlisted Actions run facts. The probe chooses
no full-ref source or trust policy. It requests no logs, artifacts, runner, or JIT.
The runner registration-token request is only the SDK's Actions-service auth.

Each phase cancels its runs and drains the queue at capacity 0 before the next.
Cleanup cancels/deletes owned runs, closes the owned PR, deletes temporary refs,
closes the session, and deletes the fixed scale set and unique group. It checks
absence and the unchanged default SHA. The closed PR record remains on GitHub.
SIGINT/SIGTERM trigger cleanup. Network loss or SIGKILL can prevent cleanup.
Inspect safe intent/status records if cleanup fails. Do not run this probe beside
another user of ciproxy-facts or facts-pr. No credentials or raw responses print.
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog=PROCEDURE.replace("WORKFLOW_REF_PROBE_SCRIPT", sys.argv[0]),
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", required=True, help="Approved private OWNER/NAME")
    parser.add_argument("--case", action="append", choices=("branch-push", "tag-push", "tag-dispatch",
                        "pull-request", "pull-request-target"), help="Repeat to select phases; default all five")
    parser.add_argument("--expected-default-sha", help="Expected full SHA or its prefix")
    parser.add_argument("--dispatch-workflow", default=".github/workflows/facts.yml")
    parser.add_argument("--pr-workflow", default=".github/workflows/facts.yml")
    parser.add_argument("--target-workflow", default=".github/workflows/facts-target.yml")
    parser.add_argument("--dispatch-jobs", type=int, default=2)
    parser.add_argument("--pr-jobs", type=int, default=3)
    parser.add_argument("--target-jobs", type=int, default=1)
    parser.add_argument("--capacity", type=int, default=8)
    parser.add_argument("--timeout-seconds", type=int, default=900)
    args = parser.parse_args()
    repository(args.repo)
    if min(args.dispatch_jobs, args.pr_jobs, args.target_jobs, args.capacity, args.timeout_seconds) <= 0:
        raise ProbeError("Job counts, capacity, and time limit must be positive")
    probe_common.SCRATCH = Path(".context/ciproxy-mvp/milestone-6/workflow-ref/tmp")
    execute(App(args.repo.split("/")[0]), args)


if __name__ == "__main__":
    entrypoint(main)
