#!/usr/bin/env python3
"""Execute an allowed job and observe a refused job through a selected runner group."""

import argparse
import os
from pathlib import Path
import uuid

from probe_common import App, ProbeError, entrypoint, repository
from probe_group_flow import execute

PROCEDURE = """
Environment:
  CIPROXY_GITHUB_APP_PATH  Required App JSON file: {"id": number-or-string, "pem": "..."}.
  GH_TOKEN / gh auth      Operator credentials for fixture refs, workflow commits,
                         run cancellation/deletion, and organization plan lookup.
                         The App keeps Actions read. Never put tokens on argv.
  RUNNER_ALLOW_RUNASROOT  Optional runner setting, passed through if already set.

Operator procedure, from the repository root:
  1. Install Python 3, OpenSSL, gh, and an unconfigured Linux Actions runner.
  2. Select two distinct private disposable repositories in one organization.
     Both must have an initial commit. Install the same App on both repositories.
     App grants: organization Self-hosted runners write, Metadata read, Actions read.
     The script checks both IDs in the App installation repository list.
  3. Authenticate gh as an operator who can write workflow files and refs in both
     repositories, cancel/delete their runs, and read the organization plan.
  4. Provide an executable launcher. It receives the JIT value only through
     ACTIONS_RUNNER_INPUT_JITCONFIG. It receives a private scratch directory through
     CIPROXY_GITHUB_PROBE_WORK_DIR. Copy the unconfigured runner there, change to
     that directory, and exec ./run.sh. Do not pass --jitconfig or print the env.
     Runner and job stdout/stderr are discarded. The script checks GitHub metadata.
  5. Export CIPROXY_GITHUB_APP_PATH, then run this command with both repositories
     and --runner-launcher. Save the safe JSON lines in .context/ if needed.
  6. Check runner_group_execution_verified and every cleanup_pass record. A result
     from Team/Enterprise or an unknown plan does not prove GitHub Free support.

Only new unique workflow branches, workflow runs, a runner group, a scale set,
a message session, and a JIT registration are changed. Existing default branches
and runner-group policies are compared before and after. Supplied repositories
are never deleted. The workflow has one 'true' step, no checkout, and no token
permissions. Existing repository-wide workflows may also run on branch creation.
Use disposable repositories whose existing workflows cannot affect other systems.
SIGINT/SIGTERM trigger cleanup. A network outage or SIGKILL can prevent cleanup.
In that case, inspect the safe created-resource records and remove only those IDs.
"""


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog=PROCEDURE,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--allowed-repo", required=True, help="Private OWNER/NAME allowed by the probe group")
    parser.add_argument("--refused-repo", required=True, help="Different private OWNER/NAME excluded by the group")
    parser.add_argument("--runner-launcher", required=True, help="Executable path; receives JIT only in its environment")
    parser.add_argument("--timeout-seconds", type=int, default=900, help="Total execution limit, default 900")
    parser.add_argument("--refused-seconds", type=int, default=60, help="Minimum queued observation after success, default 60")
    args = parser.parse_args()
    repository(args.allowed_repo)
    repository(args.refused_repo)
    org = args.allowed_repo.split("/")[0]
    if org.lower() != args.refused_repo.split("/")[0].lower() or args.allowed_repo.lower() == args.refused_repo.lower():
        raise ProbeError("Use two distinct private repositories in one organization")
    if not Path(args.runner_launcher).is_file() or not os.access(args.runner_launcher, os.X_OK):
        raise ProbeError("The runner launcher must be an existing executable file")
    if args.refused_seconds <= 0 or args.timeout_seconds <= args.refused_seconds:
        raise ProbeError("Use a positive observation window below the total execution limit")
    name = "ciproxy-probe-" + uuid.uuid4().hex
    execute(App(org), args, name)


if __name__ == "__main__":
    entrypoint(main)
