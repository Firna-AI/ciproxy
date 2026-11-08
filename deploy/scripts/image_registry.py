"""Short-lived registry authentication with no credentials in tool output."""

from contextlib import contextmanager
import os
from pathlib import Path
import re
import subprocess
import tempfile


REGISTRY = "us-central1-docker.pkg.dev"
REPOSITORY = REGISTRY + "/ciproxy/ciproxy-shared-runtime/control-plane"
DIGEST = r"sha256:[0-9a-f]{64}"


def immutable(reference):
    if not re.fullmatch(re.escape(REPOSITORY) + "@" + DIGEST, reference):
        raise ValueError("Control-plane image must be an immutable digest in the shared repository.")
    return reference


def call(*arguments, environment=None, input_text=None):
    return subprocess.run(arguments, env=environment, input=input_text,
                          text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def require(result, message):
    if result.returncode:
        raise ValueError(message)
    return result.stdout.strip()


@contextmanager
def authenticated():
    with tempfile.TemporaryDirectory(prefix="ciproxy-registry-") as directory:
        Path(directory).chmod(0o700)
        environment = dict(os.environ, DOCKER_CONFIG=directory)
        require(call("gcloud", "artifacts", "repositories", "describe", "ciproxy-shared-runtime",
                     "--location=us-central1", "--project=ciproxy", "--format=value(name)"),
                "Image repository is unavailable. Check its existence and metadata access.")
        token = require(call("gcloud", "auth", "print-access-token"),
                        "Image registry authentication failed. Check workload identity access.")
        if not token:
            raise ValueError("Image registry authentication returned no access token.")
        require(call("crane", "auth", "login", REGISTRY, "-u", "oauth2accesstoken",
                     "--password-stdin", environment=environment, input_text=token),
                "Image registry login failed. Check the repository grant.")
        yield environment


def digest(reference, environment, missing=False):
    result = call("crane", "digest", reference, environment=environment)
    unavailable = re.search(r"\b(?:401|403|429|5[0-9]{2}|DENIED|UNAUTHORIZED|FORBIDDEN)\b", result.stderr)
    if (result.returncode and missing and not unavailable
            and re.search(r"\b(?:MANIFEST_UNKNOWN|NAME_UNKNOWN)\b", result.stderr)):
        return ""
    value = require(result, "Image registry lookup failed. Check access and registry availability.")
    if not re.fullmatch(DIGEST, value):
        raise ValueError("Image registry returned an invalid SHA-256 digest.")
    return value


def lookup(source_hash, rollback):
    with authenticated() as environment:
        if rollback:
            reference = immutable(rollback)
            if digest(reference, environment) != reference.split("@", 1)[1]:
                raise ValueError("Rollback registry digest does not match the requested image.")
            return reference
        value = digest(REPOSITORY + ":source-" + source_hash, environment, missing=True)
        return REPOSITORY + "@" + value if value else ""


def archive_digest(archive):
    value = require(call("crane", "digest", "--tarball", str(archive)),
                    "Cannot read the built image archive.")
    immutable(REPOSITORY + "@" + value)
    return value


def publish(source_hash, archive, expected_digest):
    value = archive_digest(archive)
    if value != expected_digest:
        raise ValueError("Image archive differs from the build job's image digest. Start a new run.")
    reference = REPOSITORY + "@" + value
    with authenticated() as environment:
        existing = digest(REPOSITORY + ":source-" + source_hash, environment, missing=True)
        if existing:
            return REPOSITORY + "@" + existing
        require(call("crane", "push", str(archive), reference, environment=environment),
                "Image registry publish failed. Check upload access and registry availability.")
        require(call("crane", "tag", reference, "source-" + source_hash, environment=environment),
                "Image source tag publication failed. Rerun all jobs.")
        if digest(REPOSITORY + ":source-" + source_hash, environment) != value:
            raise ValueError("Published source tag does not match the built image digest.")
    return reference
