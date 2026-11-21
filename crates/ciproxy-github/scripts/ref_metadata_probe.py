#!/usr/bin/env python3
"""Measure branch/tag endpoint access with the exact three App permissions."""

import argparse
import uuid

from probe_common import App, Cleanup, ProbeError, emit, entrypoint, expect, repository, segment
from probe_fixtures import checked, default_snapshot, temporary_ref

PROCEDURE = """
Environment:
  CIPROXY_GITHUB_APP_PATH  Required App JSON file with id and pem. id may be a string.
  GH_TOKEN / gh auth      Operator credentials with Contents write for --create-temp.
                         Precreated fixtures need operator Contents read to verify them.
                         No additional App permission is requested.

Operator procedure, from the repository root:
  1. Install Python 3, OpenSSL, and gh. Authenticate gh as the fixture operator.
  2. Install the App on a private disposable repository with an initial commit.
     Grant Self-hosted runners write, Metadata read, and Actions read only.
  3. Export CIPROXY_GITHUB_APP_PATH to its JSON file path.
  4. Run --repo OWNER/NAME --create-temp. The operator creates a branch and a
     lightweight tag with one unique short name at the current default-branch SHA.
     Both refs are deleted and checked for absence on exit, also on SIGINT/SIGTERM.
     No existing ref is overwritten. The default-branch SHA is checked on exit.
  5. Alternatively use --name NAME for an existing branch/tag pair at the same SHA.
     The operator checks the pair. The script changes no precreated ref.
  6. Read the HTTP statuses and metadata_ref_access_observed. A 403 or 404 is an
     observed access result, not proof that the fixture is absent. The operator
     independently verifies both fixtures. No full-ref trust policy is selected.

Use a disposable repository whose existing push workflows cannot affect other
systems. Creating the refs can trigger such workflows. This metadata probe does
not create, cancel, or delete workflow runs. Prefer a repository with no push
workflows. Save safe JSON lines under .context/. No response body is saved.
SIGKILL or a network outage can prevent cleanup. Remove only the unique refs in
the temporary_ref_created records if a cleanup_failed record appears.
"""


def safe_fields(kind, response, name):
    if kind == "branch":
        return {"name": response.get("name"), "sha": response.get("commit", {}).get("sha")}
    rows = response if isinstance(response, list) else [response]
    return {"matching_refs": [{"ref": row.get("ref"),
                               "object_type": row.get("object", {}).get("type"),
                               "sha": row.get("object", {}).get("sha")}
                              for row in rows if row.get("ref") in ("refs/heads/" + name, "refs/tags/" + name)]}


def tag_list(app, prefix, name):
    page, matches = 1, []
    while True:
        status, response = app.call(prefix + f"/tags?per_page=100&page={page}")
        if status != 200:
            emit("ref_endpoint", endpoint="tag-list", http_status=status, page=page)
            return False
        matches.extend({"name": row["name"], "sha": row.get("commit", {}).get("sha")}
                       for row in response if row.get("name") == name)
        if len(response) < 100 or matches:
            break
        page += 1
    emit("ref_endpoint", endpoint="tag-list", http_status=200, matching_tags=matches, pages=page)
    return bool(matches)


def main():
    parser = argparse.ArgumentParser(description=__doc__, epilog=PROCEDURE,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--repo", required=True, help="Private disposable OWNER/NAME visible to the App")
    source = parser.add_mutually_exclusive_group(required=True)
    source.add_argument("--create-temp", action="store_true", help="Use operator auth to create and remove a unique pair")
    source.add_argument("--name", help="Read an existing branch and lightweight tag with this short name")
    args = parser.parse_args()
    prefix = "/repos/" + repository(args.repo)
    app = App(args.repo.split("/")[0])
    app.visible_private_repositories([args.repo])
    name = "ciproxy-ref-probe-" + uuid.uuid4().hex if args.create_temp else args.name
    with Cleanup() as cleanup:
        if args.create_temp:
            sha = default_snapshot(args.repo, cleanup)
            for namespace in ("heads", "tags"):
                temporary_ref(args.repo, namespace + "/" + name, sha, cleanup)
        refs = [checked(prefix + "/git/ref/" + namespace + "/" + segment(name))
                for namespace in ("heads", "tags")]
        if (refs[0]["object"]["sha"] != refs[1]["object"]["sha"]
                or any(ref["object"]["type"] != "commit" for ref in refs)):
            raise ProbeError("Use a branch and a lightweight tag at the same commit SHA")
        emit("operator_fixture_verified", repository=args.repo, short_name=name,
             branch=refs[0]["ref"], tag=refs[1]["ref"], sha=refs[0]["object"]["sha"])
        status, _ = app.call(prefix + "/actions/runs?per_page=1")
        expect(status, (200,), "Confirm App Actions read access on the fixture")
        results = {}
        endpoints = [
            ("branch", "/branches/" + segment(name)),
            ("head-ref", "/git/ref/heads/" + segment(name)),
            ("tag-ref", "/git/ref/tags/" + segment(name)),
            ("head-matching-refs", "/git/matching-refs/heads/" + segment(name)),
            ("tag-matching-refs", "/git/matching-refs/tags/" + segment(name)),
        ]
        for kind, path in endpoints:
            status, response = app.call(prefix + path)
            fields = safe_fields(kind, response, name) if status == 200 else {}
            emit("ref_endpoint", endpoint=kind, http_status=status, **fields)
            results[kind] = status
        tags_accessible = tag_list(app, prefix, name)
        status, rules = app.call(prefix + "/rules/branches/" + segment(name))
        emit("ref_endpoint", endpoint="branch-rules", http_status=status,
             rule_count=len(rules) if status == 200 else None,
             note="Rules can apply to a branch name that does not exist")
        for namespace in ("heads", "tags"):
            ref = "refs/" + namespace + "/" + name
            status, caches = app.call(prefix + "/actions/caches?per_page=100&ref=" + segment(ref))
            emit("ref_endpoint", endpoint="cache-metadata-" + namespace, http_status=status,
                 requested_ref=ref, total_count=caches.get("total_count") if status == 200 else None,
                 matching_refs=sorted({row["ref"] for row in caches.get("actions_caches", [])}) if status == 200 else [],
                 note="Cache keys are withheld; no cache content or archive is requested")
        emit("metadata_ref_access_observed", repository=args.repo, short_name=name,
             branch_accessible=results["branch"] == 200, tag_list_accessible=tags_accessible,
             both_full_refs_accessible=results["head-ref"] == results["tag-ref"] == 200,
             permissions="Self-hosted runners write + Metadata read + Actions read",
             trust_policy="unselected")
    emit("probe_complete", cleanup="verified" if args.create_temp else "no mutation")


if __name__ == "__main__":
    entrypoint(main)
