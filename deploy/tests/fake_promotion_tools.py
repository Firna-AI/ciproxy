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
    if action == "output" and args[-1] == "-json":
        state = Path(os.environ["FAKE_REMOTE_STATES"]).with_name("launched.json")
        print(json.dumps({"production_enabled": {"value": state.exists()}}))
        sys.exit(0)
    if action == "init":
        bucket = next((arg.split("=", 2)[2] for arg in args
                       if arg.startswith("-backend-config=bucket=")), None)
        prefix = next((arg.split("=", 2)[2] for arg in args
                       if arg.startswith("-backend-config=prefix=")), None)
        if bucket and prefix:
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
        if args[-1] == "environments":
            print(json.dumps({"main-preview": {"url_map": "ciproxy-main-preview", "deployment": {
                "project_id": "ciproxy", "region": "europe-west1",
                "cluster_name": "ciproxy-shared", "prefix": "ciproxy-main-preview",
                "namespace": "ciproxy-main-preview",
                "allowed_zones": ["europe-west1-b", "europe-west1-c", "europe-west1-d"],
                "release_secret": "fake-release-token", "runtime_repository": "example.invalid/runtime",
                "hosts": {"releases": "downloads.invalid", "dashboard": "app.invalid"},
                "buckets": {"site": "site-bucket", "dashboard": "app-bucket"},
                "control_plane_account": "runtime@example.invalid", "sql_connection": "test:eu:sql",
                "sql_user": "runtime", "kms_key": "test-key", "secrets": {},
            }}}))
        elif args[-1] == "url_map":
            print("ciproxy-main-preview")
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
elif tool == "kubectl" and args[:2] == ["get", "service"]:
    if os.environ.get("FAKE_NEG_FAIL") == "true":
        sys.exit(1)
    print(json.dumps({"metadata": {"annotations": {"cloud.google.com/neg-status": json.dumps({
        "network_endpoint_groups": {"8080": "ciproxy-main-preview-release"},
        "zones": ["europe-west1-b", "europe-west1-c", "europe-west1-d"],
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
