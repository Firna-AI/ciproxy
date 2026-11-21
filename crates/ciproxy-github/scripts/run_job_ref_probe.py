#!/usr/bin/env python3
"""Read allowlisted workflow-run and job metadata without logs or artifacts."""

import argparse
import json
from pathlib import Path

from probe_common import App, ProbeError, emit, entrypoint, expect, repository

PROCEDURE = """
Environment:
  CIPROXY_GITHUB_APP_PATH  Required App JSON file with id and pem. id may be a string.
                         App grants only Self-hosted runners write, Metadata read,
                         and Actions read. This script makes no operator gh calls.

Operator procedure, from the repository root:
  1. Install Python 3 and OpenSSL. Install the App on each private fixture repository.
     For runnable branch/tag fixtures, use push_ref_probe.py --repo OWNER/NAME.
  2. Supply existing run IDs for branch-push, tag-push, same-name-branch,
     same-name-tag, pull_request, pull_request_target, and reusable cases as available.
     Also supply target-with-nondefault-base, multiple-associated-PRs, fork-PR,
     workflow_run, or schedule cases when the fixture organization provides them.
     This script does not dispatch workflows or change any repository.
  3. Export CIPROXY_GITHUB_APP_PATH, then repeat --run CASE=OWNER/NAME:RUN_ID.
     Run IDs must be from one organization per invocation.
  4. Optional: pass --case-file PATH to a JSON array of objects. Each object has
     case, repository, run_id, and optional expected_ref and workflow_ref strings.
     Example: [{"case":"tag-push","repository":"ORG/REPO","run_id":123,
                "expected_ref":"refs/tags/example"}]
     Optional refs are operator fixture facts. They are never treated as API proof.
  5. Read each workflow_run_metadata, workflow_job_metadata, and ref_comparison.
     Save the safe JSON lines under .context/ if needed.

The probe records head_branch as returned. has_ref_namespace checks only the
refs/NAMESPACE/NAME syntax. It does not establish the ref's meaning or trust.
The probe prints top-level response field names, but withholds unknown values.
It reads scalar ref_type/tag discriminators when present. It infers no job ref
from workflow refs and selects no trigger PR from pull_requests associations.
No missing-ref fallback or trust policy is chosen. Only run/job JSON endpoints
are read. Logs, code, and artifacts are never requested. No raw response is saved.
"""

RUN_FIELDS = ("id", "event", "head_branch", "head_sha", "path", "run_attempt", "status", "conclusion")
JOB_FIELDS = ("id", "run_id", "run_attempt", "head_branch", "head_sha", "status", "conclusion",
              "runner_id", "runner_name", "runner_group_id", "runner_group_name")
REF_FIELDS = ("ref", "head_ref", "base_ref")
DISCRIMINATOR_FIELDS = ("ref_type", "head_ref_type", "is_tag", "is_branch", "tag")


def select(row, fields):
    return {key: row[key] for key in fields if key in row}


def has_ref_namespace(value):
    parts = value.split("/", 2) if isinstance(value, str) else []
    return len(parts) == 3 and parts[0] == "refs" and bool(parts[1]) and bool(parts[2])


def discriminators(row):
    return {key: value for key, value in select(row, DISCRIMINATOR_FIELDS).items()
            if value is None or isinstance(value, (str, bool, int, float))}


def observed_fields(row, fields):
    return {**select(row, fields), **discriminators(row), "top_level_fields": sorted(row),
            "has_ref_namespace": has_ref_namespace(row.get("head_branch"))}


def safe_repo(row):
    return None if row is None else select(row, ("id", "full_name", "private"))


def safe_pr(row):
    result = select(row, ("number", "id"))
    for key in ("head", "base"):
        source = row.get(key) or {}
        result[key] = {**select(source, ("ref", "sha")), "repository": safe_repo(source.get("repo"))}
    return result


def read_case(app, case):
    prefix = "/repos/" + repository(case["repository"]) + "/actions/runs/" + str(case["run_id"])
    status, run = app.call(prefix)
    expect(status, (200,), "Read workflow-run metadata with App Actions read")
    run_fields = observed_fields(run, RUN_FIELDS + REF_FIELDS)
    run_fields["github_event"] = run_fields.pop("event", None)
    run_fields.update({"repository": safe_repo(run.get("repository")),
                       "head_repository": safe_repo(run.get("head_repository")),
                       "pull_requests": [safe_pr(row) for row in (run.get("pull_requests") or [])],
                       "referenced_workflows": [select(row, ("path", "ref", "sha"))
                                                for row in (run.get("referenced_workflows") or [])]})
    emit("workflow_run_metadata", case=case["case"], **run_fields)
    page, job_refs, count = 1, [], 0
    while True:
        status, response = app.call(prefix + f"/jobs?filter=all&per_page=100&page={page}")
        expect(status, (200,), "Read workflow-job metadata with App Actions read")
        batch = response["jobs"]
        for job in batch:
            fields = observed_fields(job, JOB_FIELDS + REF_FIELDS)
            emit("workflow_job_metadata", case=case["case"], **fields)
            job_refs.append({"job_id": job["id"], "direct_ref_fields": select(job, REF_FIELDS),
                             "ref_discriminators": discriminators(job), "head_branch": job.get("head_branch"),
                             "has_ref_namespace": has_ref_namespace(job.get("head_branch"))})
            count += 1
        if len(batch) < 100:
            break
        page += 1
    comparison = {"run_direct_ref_fields": select(run, REF_FIELDS), "job_direct_ref_fields": job_refs,
                  "head_branch": run.get("head_branch"), "has_ref_namespace": has_ref_namespace(run.get("head_branch")),
                  "run_ref_discriminators": discriminators(run), "job_ref_inference_performed": False,
                  "workflow_ref_inference_performed": False, "trigger_pr_selection_performed": False,
                  "job_count": count, "trust_policy": "unselected"}
    comparison.update(select(case, ("expected_ref", "workflow_ref")))
    emit("ref_comparison", case=case["case"], **comparison)


def load_cases(args):
    cases = []
    if args.case_file:
        try:
            cases = json.loads(Path(args.case_file).read_text())
            if not isinstance(cases, list):
                raise ValueError()
        except (OSError, ValueError):
            raise ProbeError("Case file must be a JSON array; contents withheld") from None
    for value in args.run:
        try:
            label, target = value.split("=", 1)
            repo, run_id = target.rsplit(":", 1)
            cases.append({"case": label, "repository": repo, "run_id": int(run_id)})
        except ValueError:
            raise ProbeError("Use --run CASE=OWNER/NAME:RUN_ID") from None
    if not cases:
        raise ProbeError("Supply at least one --run or a nonempty --case-file")
    labels, organizations = set(), set()
    for case in cases:
        try:
            repository(case["repository"])
            valid = isinstance(case["case"], str) and bool(case["case"]) and int(case["run_id"]) > 0
            if not valid or case["case"] in labels:
                raise ValueError()
            for key in ("expected_ref", "workflow_ref"):
                if key in case and not isinstance(case[key], str):
                    raise ValueError()
            case["run_id"] = int(case["run_id"])
            labels.add(case["case"])
            organizations.add(case["repository"].split("/")[0].lower())
        except (KeyError, TypeError, ValueError):
            raise ProbeError("Each case needs a unique name, OWNER/NAME, and positive run_id") from None
    if len(organizations) != 1:
        raise ProbeError("Use one organization per run/job probe invocation")
    return cases, organizations.pop()


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog=PROCEDURE,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--run", action="append", default=[], help="Repeat CASE=OWNER/NAME:RUN_ID")
    parser.add_argument("--case-file", help="Optional JSON array with safe operator fixture facts")
    args = parser.parse_args()
    cases, org = load_cases(args)
    app = App(org)
    app.visible_private_repositories([case["repository"] for case in cases])
    for case in cases:
        read_case(app, case)
    emit("probe_complete", cases=len(cases), mutations=0, logs_read=0, artifacts_read=0)


if __name__ == "__main__":
    entrypoint(main)
