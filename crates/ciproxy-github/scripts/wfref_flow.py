"""Five runner-free workflow-ref cases over the owner-approved facts fixtures."""

import json
import time
import uuid

from probe_common import Cleanup, ProbeError, emit, expect, segment
from probe_group_flow import create_group, group_snapshot
from probe_scales import SCALE_PATH
from wfref_queue import Observer, RunnerFreeScale
from wfref_resources import LABEL, PR_HEAD, Resources


def check_label_unused(app, service):
    for group in group_snapshot(app):
        status, response = service.call(SCALE_PATH + f"?runnerGroupId={group['id']}&name={LABEL}")
        expect(status, (200,), "Check that the approved fixed scale-set name is unused")
        if response.get("count") != 0:
            raise ProbeError("An existing ciproxy-facts scale set blocks this probe")
    page = 1
    while True:
        status, response = app.call(f"/orgs/{segment(app.org)}/actions/runners?per_page=100&page={page}")
        expect(status, (200,), "Check for an existing runner with the approved fixed label")
        for runner in response["runners"]:
            if any(label.get("name", "").lower() == LABEL for label in runner.get("labels", [])):
                raise ProbeError("An existing ciproxy-facts runner blocks this runner-free probe")
        if len(response["runners"]) < 100:
            return
        page += 1


def push_workflow(name, branch_case):
    branches = f"    branches: [{json.dumps(name)}]\n" if branch_case else "    branches-ignore: ['**']\n"
    return (f"name: {name}\non:\n  push:\n" + branches + f"    tags: [{json.dumps(name)}]\n"
            "permissions: {}\njobs:\n  probe:\n"
            f"    runs-on: {LABEL}\n    steps:\n      - run: 'true'\n")


def execute(app, args):
    selected = set(args.case or ("branch-push", "tag-push", "tag-dispatch", "pull-request", "pull-request-target"))
    needs_pr = bool(selected & {"pull-request", "pull-request-target"})
    nonce = "ciproxy-wfref-" + uuid.uuid4().hex
    visible = app.visible_private_repositories([args.repo])
    deadline = time.monotonic() + args.timeout_seconds
    with Cleanup() as cleanup:
        resources = Resources(app, args.repo, nonce, cleanup, needs_pr, args.expected_default_sha)
        if "tag-dispatch" in selected:
            resources.fixture(args.dispatch_workflow)
        if needs_pr:
            resources.fixture(args.pr_workflow)
            resources.fixture(args.target_workflow)
        service = RunnerFreeScale(app)
        check_label_unused(app, service)
        group_id = create_group(app, nonce, visible[args.repo.lower()]["id"], cleanup)
        service.create(group_id, LABEL, cleanup)
        service.open(nonce, cleanup)
        observer = Observer(app, service, resources, args.capacity)

        def phase(items):
            for item in items:
                resources.wait(lambda: resources.discover(item), deadline, item["name"] + " run")
            observer.collect(items, deadline)
            observer.cancel_and_drain(items, deadline)

        if selected & {"branch-push", "tag-push"}:
            workflow = ".github/workflows/" + nonce + ".yml"
            resources.branch(nonce)
            branch = resources.intent("branch-push", "push", workflow, 1, branches=[nonce]) if "branch-push" in selected else None
            sha = resources.write(nonce, workflow, push_workflow(nonce, "branch-push" in selected))
            if branch is not None:
                branch["shas"] = {sha}
                phase([branch])
            if "tag-push" in selected:
                tag = resources.intent("same-name-tag-push", "push", workflow, 1, [sha], [nonce])
                resources.tag(nonce, sha)
                phase([tag])
        if "tag-dispatch" in selected:
            tag_name = nonce + "-dispatch"
            resources.tag(tag_name, resources.default_sha)
            dispatch = resources.intent("workflow-dispatch-on-tag", "workflow_dispatch", args.dispatch_workflow,
                                        args.dispatch_jobs, [resources.default_sha], [tag_name])
            resources.dispatch(args.dispatch_workflow, tag_name)
            phase([dispatch])
        if needs_pr:
            pr = resources.intent("pull_request", "pull_request", args.pr_workflow, args.pr_jobs, branches=[PR_HEAD])
            target = resources.intent("pull_request_target", "pull_request_target", args.target_workflow,
                                      args.target_jobs, branches=[PR_HEAD, resources.default_branch])
            resources.open_pr()
            pr["shas"] = {resources.pr_sha}
            target["shas"] = {resources.pr_sha, resources.default_sha}
            phase([pr, target])
            resources.close_pr()
        emit("workflow_ref_cases_observed", cases=[item["name"] for item in resources.intents if item["run_id"]],
             runner_launches=0, jit_configurations=0, policy="unselected")
    emit("probe_complete", cleanup="verified", closed_pr_records_retained=needs_pr)
