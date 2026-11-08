"""Read real workflow configuration and evaluate its emitted claim policy."""

import ast
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

from deploy.tests.promotion_support import ROOT


PROTECTED = "Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@refs/heads/main"
SOURCE_SHA = "c" * 40
TOOLS_SHA = "b" * 40


def workflows(root=None):
    root = root or ROOT
    paths = sorted((root / ".github/workflows").glob("*.yml"))
    output = subprocess.check_output(
        ["node", str(ROOT / "deploy/tests/read_workflows.cjs"), *map(str, paths)], text=True)
    return {Path(path).name: value for path, value in json.loads(output).items()}


def providers(root=None):
    root = root or ROOT
    scratch = ROOT / ".context/control-plane-image/fix-round-1/tests"
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=scratch) as temporary:
        directory = Path(temporary)
        source = root / "deploy/terraform/bootstrap"
        for path in source.iterdir():
            if path.suffix == ".tf" or path.name == ".terraform.lock.hcl":
                shutil.copyfile(path, directory / path.name)
        (directory / ".terraform").mkdir()
        (directory / ".terraform/providers").symlink_to(
            (ROOT / "deploy/terraform/bootstrap/.terraform/providers").resolve(), target_is_directory=True)
        (directory / "claims.tftest.hcl").write_text('''mock_provider "google" {}
variables {
  project_id = "ciproxy"
  production_enabled = false
}
run "read_claim_conditions" { command = plan }
''')
        output = subprocess.check_output(
            ["terraform", f"-chdir={directory}", "test", "-json", "-verbose"], text=True)
    plan = next(json.loads(line)["test_plan"] for line in output.splitlines() if '"test_plan":' in line)
    return {change["index"]: change["change"]["after"]
            for change in plan["resource_changes"]
            if change["type"] == "google_iam_workload_identity_pool_provider"
            and str(change.get("index", "")).startswith("image_")}


def admits(provider, claims, oidc=True):
    if not oidc:
        return False
    expression = provider["attribute_condition"].replace("&&", " and ").replace("||", " or ")
    tree = ast.parse(expression.strip(), mode="eval")
    permitted = (ast.Expression, ast.BoolOp, ast.And, ast.Or, ast.Compare, ast.Eq, ast.In,
                 ast.BinOp, ast.Add, ast.Attribute, ast.Name, ast.Load, ast.Constant, ast.List)
    if any(not isinstance(node, permitted) for node in ast.walk(tree)):
        raise ValueError("The claim test needs support for this CEL expression.")
    class Claims:
        def __getattr__(self, name):
            return claims.get(name, "")
    return eval(compile(tree, "<provider CEL equality subset>", "eval"),
                {"__builtins__": {}}, {"assertion": Claims()})


def jobs(documents):
    entry = documents["deploy-main-preview.yml"]
    result = []
    caller = "Firna-AI/ciproxy/.github/workflows/deploy-main-preview.yml@refs/heads/calummoore/ci-proxy-load-balancer"
    for name, job in entry["jobs"].items():
        if job.get("uses", "").startswith("Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@"):
            reference = job["uses"].replace("@main", "@refs/heads/main")
            called = documents.get("deploy-control-plane-image.yml", {})
            for child, configuration in called.get("jobs", {}).items():
                result.append((child, configuration, reference, called.get("permissions", {})))
        elif "steps" in job:
            result.append((name, job, caller, entry.get("permissions", {})))
    return result


def image_jobs(documents):
    return [item for item in jobs(documents) if any(
        "GCP_IMAGE_" in str(step.get("with", {}).get("service_account", ""))
        for step in item[1].get("steps", []))]


def claims(job, reusable=None):
    result = {"repository_id": "1442045837", "repository_owner_id": "307519338",
              "environment": job.get("environment", ""), "event_name": "push",
              "ref": "refs/heads/calummoore/ci-proxy-load-balancer",
              "workflow_ref": "Firna-AI/ciproxy/.github/workflows/deploy-main-preview.yml@refs/heads/calummoore/ci-proxy-load-balancer"}
    if reusable:
        result["job_workflow_ref"] = reusable
        result["job_workflow_sha"] = TOOLS_SHA
    return result


def resolve(expression, context):
    match = re.fullmatch(r"\$\{\{\s*([\w.-]+)\s*\}\}", expression or "")
    if not match:
        return expression
    value = context
    for key in match[1].split("."):
        value = value.get(key, {})
    return value
