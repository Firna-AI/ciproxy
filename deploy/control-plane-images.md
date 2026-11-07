# Control-plane images

Each accepted preview push builds or reuses the control-plane image.
Preview deploys its immutable `@sha256:` reference. Production deploys the
image recorded by the successful upstream preview workload. Production
neither builds nor publishes an image. Runtime readiness remains a separate
environment setting. Image publication does not enable the runtime.

## Build and reuse

The image lookup first verifies the repository with an authenticated,
read-only metadata request. An absent repository or failed request stops
the run. A
`source-<SHA-256>` tag selects the image for the exact build inputs. A
missing manifest starts a build. A denied request, missing repository, or
unavailable registry stops the run. Those errors never count as a cache miss.

The source checkout names the exact push commit. Every image job uses the
same immutable trusted tools commit from that accepted push. Trusted scripts
run outside the separate source checkout. The build runner has only
`contents: read`. It has no cloud identity, environment, or OIDC permission.
Pinned BuildKit builds a Linux AMD64 Docker archive. The workflow artifact
expires after one day. The separate publisher authenticates through WIF,
reads the archive with pinned crane, pushes its digest, and adds the source
tag. It never executes the archive or scripts from the source checkout.

The source hash covers the paths, modes, and bytes of the selected tracked
files. It covers `.dockerignore`, the Dockerfile, root Cargo inputs,
toolchain, crate manifests, Rust sources, build scripts, migrations, and
copied task-runner sources. It also covers the target platform and trusted
build scripts, installer pins, and workflow configuration.

One selector supplies both hashing and the actual build context. It reads
the ordered `.dockerignore` patterns. Positive patterns ending in `/` open
directory traversal only. Positive file patterns select files. Negative
patterns remove files and their descendants. This makes the source-only
intent explicit: Docker itself ignores trailing slashes and would otherwise
also admit crate documentation. New positive file patterns reach both the
hash and the build. Escaped patterns need selector support before use.
Untracked files never enter the context. Excluded documentation, secrets,
and local output change neither the context nor its hash.

The image revision label records the original build commit. A reused image
keeps that label. The runtime `BUILD_ID` still records the deploy commit.
The current Dockerfile uses mutable base tags and Debian package indexes.
The source tag identifies the first published image for those inputs. It
does not promise a reproducible rebuild after base packages change.

Registry tokens enter crane through standard input. Each authenticated
operation creates a private temporary `DOCKER_CONFIG` and removes it on
success or failure. Commands suppress registry authentication payloads.

## Install the repository grants

Use the operator account and a reviewed checkout of this deployment code.
Complete [initial bootstrap and shared core](runtime-identity.md) first.
Initial bootstrap works before the repository exists. It defaults
`control_plane_registry_ready` to false. After shared core creates the
repository, the operator enables its image grants:

```sh
gcloud auth application-default login
gcloud auth application-default set-quota-project ciproxy
gcloud artifacts repositories describe ciproxy-shared-runtime \
  --location=us-central1 --project=ciproxy
bash deploy/scripts/bootstrap.sh main-preview --existing-state --image-registry
```

Review the Terraform apply before accepting it. This is the same upgrade
command for an existing deployment. It reads remote bootstrap outputs and
preserves the production phase and captured runtime account UIDs. Later
operator retries and protected-main CI plans preserve the registry flag.
The primary preview branch cannot apply bootstrap.

Bootstrap creates these accounts and grants:

| Account | Grant |
| --- | --- |
| `ciproxy-image-lookup` | Custom `ciproxyImageLookup`: `artifactregistry.repositories.get` and `artifactregistry.repositories.downloadArtifacts` on `ciproxy-shared-runtime` only. |
| `ciproxy-image-publish` | Custom `ciproxyImagePublish`: `artifactregistry.repositories.get`, `artifactregistry.repositories.downloadArtifacts`, and `artifactregistry.repositories.uploadArtifacts` on that repository only. |
| Both image accounts | `roles/iam.workloadIdentityUser` on the corresponding account for its separate GitHub pool and access attribute. |
| `ciproxy-trust-preview-apply`, `ciproxy-trust-prod-apply` | Custom `ciproxyImageRepositoryPolicy`: `artifactregistry.repositories.getIamPolicy` and `artifactregistry.repositories.setIamPolicy` on that repository only. |

The trust grant lets later protected-main bootstrap plans maintain these
repository bindings. It grants no image upload or deletion. Central plan
already has repository-policy metadata read access. No new plan role is
needed. Preview-accessible shared apply has no repository policy writer.

Both image providers require repository ID `1442045837`, owner ID
`307519338`, environment `main-preview`, event `push`, and the exact preview
workflow. They accept the primary branch and `main`. They remain available
after launch because preview must publish before production can promote.
The build runner receives none of these grants. Production receives no
image publisher. Existing node image-reader rights remain unchanged.
These image roles contain no image deletion or IAM policy permission.
Google lists the required Docker upload/download permissions in its
[Artifact Registry audit reference][docker-iam].

Create the `main-preview` GitHub environment as
[setup](setup.md#4-set-github-variables) directs. Then set its four new public
GitHub variables:

```sh
terraform -chdir=deploy/terraform/bootstrap output -json github_environment_variables \
  | jq -r '."main-preview" | to_entries[] | select(.key | startswith("GCP_IMAGE_")) | [.key, .value] | @tsv' \
  | while IFS=$'\t' read -r name value; do
      gh variable set "$name" --repo=Firna-AI/ciproxy \
        --env=main-preview --body="$value"
    done
```

The names are `GCP_IMAGE_LOOKUP_WIF_PROVIDER`,
`GCP_IMAGE_LOOKUP_SERVICE_ACCOUNT`, `GCP_IMAGE_PUBLISH_WIF_PROVIDER`, and
`GCP_IMAGE_PUBLISH_SERVICE_ACCOUNT`. Do not set them in production.

## Remove the legacy image variables

Save the old public values for recovery, then remove `CONTROL_PLANE_IMAGE`
from both GitHub environments. Normal workflows ignore that variable.
Keep `CONTROL_PLANE_RUNTIME_READY` and the other runtime settings unchanged.

```sh
mkdir -p .context/control-plane-image
for environment in main-preview production; do
  gh variable list --repo=Firna-AI/ciproxy --env="$environment" \
    --json name,value > ".context/control-plane-image/previous-$environment.json"
  if jq -e 'any(.[]; .name == "CONTROL_PLANE_IMAGE")' \
    ".context/control-plane-image/previous-$environment.json" >/dev/null; then
    gh variable delete CONTROL_PLANE_IMAGE --repo=Firna-AI/ciproxy \
      --env="$environment"
  fi
done
```

Leave `CONTROL_PLANE_ROLLBACK_IMAGE` unset for normal operation. Select a
push run containing this workflow version, then rerun all jobs:

```sh
gh run list --repo=Firna-AI/ciproxy --workflow=deploy-main-preview.yml \
  --branch=calummoore/ci-proxy-load-balancer
gh run rerun RUN_ID --repo=Firna-AI/ciproxy
```

Check the selected digest in the image job output and the completed
`workload-main-preview-<attempt>` artifact. Production downloads that
exact upstream run and attempt. It rejects an absent image, another
repository, a moving tag, or a mismatched evidence binding. An older
preview artifact without the image field requires a fresh preview run.

## Roll back and resume

Select an existing known-good digest in the shared repository. Set this
explicit preview override, then rerun all jobs of a trusted preview push:

```sh
ROLLBACK_IMAGE=us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime/control-plane@sha256:DIGEST
gh variable set CONTROL_PLANE_ROLLBACK_IMAGE --repo=Firna-AI/ciproxy \
  --env=main-preview --body="$ROLLBACK_IMAGE"
gh run rerun RUN_ID --repo=Firna-AI/ciproxy
```

Lookup checks that this digest exists. Preview keeps current runtime
settings and secret selectors while deploying that image. Its completed
workload artifact records the rollback digest. After launch, use a preview
push run on `main` to promote that same digest into production.
Production ignores its own image and rollback variables.

The override remains active until you remove it. Resume automatic source
selection with these commands:

```sh
gh variable delete CONTROL_PLANE_ROLLBACK_IMAGE --repo=Firna-AI/ciproxy \
  --env=main-preview
gh run rerun RUN_ID --repo=Firna-AI/ciproxy
```

Keep the chosen image in Artifact Registry for the required rollback
period. The one-day workflow archive is only a build-to-publish transfer.
For a failed publish or expired transfer, rerun all jobs. A completed
source tag reuses its existing digest. A missing tag rebuilds from the
exact source. A failed tag write can leave an untagged digest. That digest
is safe to select explicitly for recovery after operator inspection.

[docker-iam]: https://cloud.google.com/artifact-registry/docs/audit-logging
