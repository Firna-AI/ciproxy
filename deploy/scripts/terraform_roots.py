"""Root instances and source files used by Terraform promotion."""

import hashlib
from pathlib import Path


ENVIRONMENTS = {"main-preview": "internal-498318", "production": "ciproxy"}
ROOTS = ("seed", "bootstrap", "foundation", "edge")
REPOSITORY = "Firna-AI/ciproxy"
PRIMARY = "refs/heads/calummoore/ci-proxy-load-balancer"


def directory(environment, root):
    """Return the source directory of a supported root instance."""
    if environment not in ENVIRONMENTS or root not in ROOTS:
        raise ValueError("Use main-preview or production and seed, bootstrap, foundation or edge.")
    if root in ("seed", "bootstrap"):
        return Path("deploy/terraform") / root
    base = Path("deploy/environments") / environment
    return base / "edge" if root == "edge" else base


def scope(environment, root):
    directory(environment, root)
    return f"ciproxy/{environment}/{root}"


def approval_path(repository, environment, root):
    directory(environment, root)
    return repository / "deploy/approvals" / environment / f"{root}.json"


def configuration_digest(repository):
    """Bind an approval to the full Terraform configuration, including locks."""
    sources = (repository / "deploy/terraform", repository / "deploy/environments")
    files = sorted(
        path for source in sources for path in source.rglob("*")
        if path.is_file() and ".terraform" not in path.parts
        and (path.suffix == ".tf" or path.name.endswith(".tf.json")
             or path.name == ".terraform.lock.hcl")
    )
    if not files:
        raise ValueError("Terraform configuration files are missing.")
    digest = hashlib.sha256()
    for path in files:
        digest.update(path.relative_to(repository).as_posix().encode())
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return f"sha256:{digest.hexdigest()}"


def trust_resource(root, resource):
    """CI must never remove its own state bucket, identities, or access grants."""
    if root == "seed":
        return resource.get("type") == "google_storage_bucket"
    if root != "bootstrap":
        return False
    types = {
        "google_service_account", "google_service_account_iam_member",
        "google_iam_workload_identity_pool", "google_iam_workload_identity_pool_provider",
        "google_project_iam_custom_role", "google_project_iam_member",
        "google_storage_bucket_iam_member",
    }
    return resource.get("type") in types or any(
        isinstance(resource.get(key), str)
        and any(f".{kind}." in "." + resource[key] for kind in types)
        for key in ("address", "previous_address")
    )
