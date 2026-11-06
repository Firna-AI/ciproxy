"""Install provider command doubles at the CLI boundary."""

import os
import sys


PROGRAM = r'''
import json
import os
from pathlib import Path
import sys

tool = Path(sys.argv[0]).name
args = sys.argv[1:]
record = {"tool": tool, "args": args}
if "--input" in args:
    record["payload"] = json.loads(Path(args[args.index("--input") + 1]).read_text())
if tool == "terraform" and len(args) > 1 and args[1] == "apply":
    variable_file = next((arg.split("=", 1)[1] for arg in args if arg.startswith("-var-file=")), None)
    if variable_file:
        record["variables"] = json.loads(Path(variable_file).read_text())
if tool == "kubectl" and args[:1] == ["apply"]:
    record["manifest"] = json.loads(sys.stdin.read())
with open(os.environ["FAKE_COMMAND_LOG"], "a") as log:
    log.write(json.dumps(record) + "\n")
if tool == "gh":
    state = Path(os.environ["FAKE_RULESETS"])
    if "--method" not in args:
        print(json.dumps([json.loads(state.read_text())]))
    else:
        state.write_text(json.dumps([{"id": 42, "name": record["payload"]["name"], "target": "branch"}]))
elif tool == "terraform":
    root = Path(args[0].split("=", 1)[1])
    action = args[1]
    selected = root / ".terraform/current-prefix"
    prefix = selected.read_text() if selected.exists() else "main-preview/data"
    if action == "apply" and prefix == os.environ.get("FAKE_FAIL_EDGE_PREFIX"):
        print("Error 404: preview networkEndpointGroups/NEG was not found")
        sys.exit(1)
    if action == "plan":
        Path(next(arg[5:] for arg in args if arg.startswith("-out="))).write_bytes(b"SAVED-BINARY")
        print("Hand-worked saved plan")
        sys.exit(int(os.environ.get("FAKE_PLAN_STATUS", "2")))
    if action == "show":
        print(Path(os.environ["FAKE_PLAN_JSON"]).read_text())
    if action == "apply" and "seed-local" in str(root):
        (root / "terraform.tfstate").write_text('{"version":4,"serial":1}')
    if action == "apply" and "-var=production_enabled=true" in args:
        Path(os.environ["FAKE_REMOTE_STATES"]).with_name("launched.json").write_text("true")
    if action == "apply" and "variables" in record and "runtime_service_account_uids" in record["variables"]:
        Path(os.environ["FAKE_REMOTE_STATES"]).with_name("runtime-uids.json").write_text(json.dumps(record["variables"]["runtime_service_account_uids"]))
    if action == "output" and args[-1] == "-json":
        state = Path(os.environ["FAKE_REMOTE_STATES"]).with_name("launched.json")
        uids = state.with_name("runtime-uids.json")
        print(json.dumps({"production_enabled": {"value": state.exists()}, "runtime_service_account_uids": {"value": json.loads(uids.read_text()) if uids.exists() else {}}}))
        sys.exit(0)
    if action == "init":
        bucket = next((arg.split("=", 2)[2] for arg in args
                       if arg.startswith("-backend-config=bucket=")), None)
        prefix = next((arg.split("=", 2)[2] for arg in args
                       if arg.startswith("-backend-config=prefix=")), None)
        if bucket and prefix:
            selected.parent.mkdir(parents=True, exist_ok=True)
            selected.write_text(prefix)
            store = Path(os.environ["FAKE_REMOTE_STATES"])
            states = json.loads(store.read_text())
            initial = ((root / "terraform.tfstate").read_text() if "-migrate-state" in args
                       else '{"version":4,"serial":0}')
            states.setdefault(f"gs://{bucket}/{prefix}/default.tfstate", initial)
            store.write_text(json.dumps(states))
        if prefix and prefix.endswith("/seed"):
            backend = root / ".terraform/terraform.tfstate"
            backend.parent.mkdir(parents=True, exist_ok=True)
            backend.write_text('{"backend":{"type":"gcs"}}')
    if action == "output":
        if args[-1] == "deployment":
            print(json.dumps({
                "project_id": "ciproxy", "region": "us-central1",
                "cluster_name": "ciproxy-shared", "prefix": "ciproxy-main-preview",
                "namespace": "ciproxy-main-preview",
                "allowed_zones": ["us-central1-a", "us-central1-b", "us-central1-c"],
                "release_secret": "fake-release-token", "runtime_repository": "example.invalid/runtime",
                "hosts": {"releases": "downloads.invalid", "dashboard": "app.invalid"},
                "buckets": {"site": "site-bucket", "dashboard": "app-bucket"},
                "control_plane_account": "runtime@example.invalid", "sql_connection": "test:eu:sql",
                "sql_user": "runtime", "kms_key": "test-key", "secrets": {},
            }))
        elif args[-1] == "url_map":
            print("ciproxy-production" if prefix.startswith("production/") else "ciproxy-main-preview")
        else:
            print("{}")
elif tool == "gcloud" and args[:3] == ["iam", "service-accounts", "describe"]:
    state = Path(os.environ["FAKE_REMOTE_STATES"]).with_name("launched.json")
    disabled = os.environ.get("FAKE_PRODUCTION_READY", "false") == "true" or state.exists()
    if args[3].startswith("ciproxy-trust-preview") and "FAKE_TRUST_DISABLED" in os.environ:
        disabled = os.environ["FAKE_TRUST_DISABLED"] == "true"
    print(json.dumps({"disabled": disabled}))
elif tool == "gcloud" and args[:3] == ["secrets", "versions", "access"]:
    print("fake-token")
elif tool == "kubectl" and args[:2] == ["get", "serviceaccount"]:
    print(os.environ.get("FAKE_KSA_UID", "22222222-2222-4222-8222-222222222222" if "--namespace=ciproxy-production" in args else "11111111-1111-4111-8111-111111111111"))
elif tool == "kubectl" and args[:2] == ["get", "service"]:
    if os.environ.get("FAKE_NEG_FAIL") == "true":
        sys.exit(1)
    suffix = "control" if args[2] == "control-plane-neg" else "release"
    print(json.dumps({"metadata": {"annotations": {"cloud.google.com/neg-status": json.dumps({
        "network_endpoint_groups": {"8080": "ciproxy-main-preview-" + suffix},
        "zones": ["us-central1-a", "us-central1-b", "us-central1-c"],
    })}}}))
elif tool == "python3" and args[:1] != ["deploy/scripts/publish_static.py"]:
    os.execv(os.environ["FAKE_REAL_PYTHON"], [os.environ["FAKE_REAL_PYTHON"], *args])
'''


def install(case):
    tools = case.root / ".context/bin"
    tools.mkdir()
    for name in ("terraform", "gcloud", "kubectl", "helm", "python3", "gh"):
        path = tools / name
        path.write_text(f"#!{sys.executable}\n" + PROGRAM)
        path.chmod(0o755)
    log = case.root / ".context/commands.jsonl"
    log.write_text("")
    state = case.root / ".context/rulesets.json"
    state.write_text("[]")
    remote_states = case.root / ".context/remote-states.json"
    remote_states.write_text("{}")
    case.env.update({
        "PATH": str(tools) + os.pathsep + case.env["PATH"],
        "FAKE_REAL_PYTHON": sys.executable,
        "FAKE_COMMAND_LOG": str(log), "FAKE_PLAN_JSON": str(case.plan),
        "FAKE_RULESETS": str(state),
        "FAKE_REMOTE_STATES": str(remote_states),
    })
    return log
