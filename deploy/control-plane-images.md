# Control-plane images

Each accepted preview push builds or reuses the control-plane image.
An enabled preview deploys its immutable `@sha256:` reference. Production
uses the image recorded after that preview rollout succeeds. Production
neither builds nor publishes an image. Runtime readiness remains a separate
environment setting. Image publication does not enable the runtime.

## Build and reuse

The image lookup first verifies the repository with an authenticated,
read-only metadata request. An absent repository or failed request stops
the run. A
`source-<SHA-256>` tag selects the image for the exact build inputs. A
missing manifest starts a build. A denied request, missing repository, or
unavailable registry stops the run. Those errors never count as a cache miss.

Until launch, the caller uses the reusable image workflow on the primary
branch. Both image identities require that exact `job_workflow_ref`.
[Image workflow trust](image-workflow-bootstrap.md) gives the claim, its
activation, and its move to protected `main` at launch. An ordinary caller
job cannot obtain either image identity, including the npm workload job.
The reusable workflow reads its own `job_workflow_sha` from GitHub OIDC
before checking out any repository. It prints no token or claim payload.
Every image job checks out tools at that immutable trusted revision.
The source checkout separately names the exact push commit. Trusted scripts
run outside that source checkout. The build runner has only
`contents: read`. It has no cloud identity, environment, or OIDC permission.
Pinned BuildKit builds a Linux AMD64 Docker archive. The workflow artifact
expires after one day. Build records its immutable artifact ID and the
canonical image digest from `crane digest --tarball`. Publish downloads that
exact artifact ID. It compares the archive's canonical digest with the
build output before any registry write. An archive found only by name is
not publication authority. The separate publisher uses WIF, pushes the
digest, and adds the source tag. It never executes the archive or source
scripts. Cached images skip both the build and publisher credentials.

The launcher creates a short runner-owned `0700` directory for its Unix
socket and private daemon state. It passes that address and the runner's
primary group directly to BuildKit. Readiness uses `BUILDKIT_HOST` as the
ordinary runner user. The launcher writes the same address to `GITHUB_ENV`
for later build steps. It does not use or change the global BuildKit socket.
Logs remain under `.context/control-plane-image/`. A failed start reports
the local connection cause without printing the raw error payload.

For a manual smoke, keep startup and clients in one shell execution.
Source the generated environment file after the launcher returns:

```sh
bash deploy/scripts/start-buildkit.sh
. .context/control-plane-image/buildkit-env.sh
buildctl debug workers
# Run the image build with this same BUILDKIT_HOST.
```

The file also sets `CIPROXY_BUILDKIT_PID` and
`CIPROXY_BUILDKIT_DIRECTORY`. After the smoke, stop only that daemon with
`sudo -n kill "$CIPROXY_BUILDKIT_PID"`. Its private state is inside that
directory. Existing global services stay untouched.

The source hash covers the paths, modes, and bytes of the selected tracked
files. It covers `.dockerignore`, the Dockerfile, root Cargo inputs,
toolchain, crate manifests, Rust sources, build scripts, migrations, and
copied task-runner sources. It also covers the target platform and trusted
build scripts, installer pins, and reusable workflow configuration.
Production promotion code and caller workflow changes do not change this
hash.

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

[Activation](image-workflow-bootstrap.md#activate-the-primary-binding)
gives the exact operator steps. An operator bootstrap installs these
grants and binds both image providers to the trusted workflow. Activation
then sets the image variables. There is no caller-branch fallback. Initial
bootstrap works before the repository exists. It defaults
`control_plane_registry_ready` to false. Activation enables the image
grants after shared core creates the repository.

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
caller workflow. They also require the one reusable-workflow claim that
[image workflow trust](image-workflow-bootstrap.md) gives, whatever the
caller's branch. The providers map `attribute.job_workflow_ref` from the
signed token. Their repository roles remain unchanged. They accept primary
and `main` callers. They remain available after launch because preview
must publish before production can promote.
The build runner receives none of these grants. Production receives no
image publisher. Existing node image-reader rights remain unchanged.
These image roles contain no image deletion or IAM policy permission.
Google lists the required Docker upload/download permissions in its
[Artifact Registry audit reference][docker-iam].

Activation sets four public GitHub variables in the `main-preview`
environment: `GCP_IMAGE_LOOKUP_WIF_PROVIDER`,
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

## Check the image run

Leave `CONTROL_PLANE_ROLLBACK_IMAGE` unset for normal operation. Inspect
the [activation rerun](image-workflow-bootstrap.md#activate-the-primary-binding)
or a later accepted preview push.
Check the selected digest in the image job output. An enabled rollout saves
its deployed image only after Helm and all NEG waits pass. A disabled
preview saves an empty image. Check the completed
`workload-main-preview-<attempt>` artifact. Production downloads that
exact upstream run and attempt. It rejects another repository, a moving
tag, or a mismatched evidence binding. An enabled
production runtime rejects the empty image through its existing runtime
validation. A disabled production runtime can still deploy static content
and the release server. Enabling production requires a successful enabled
preview rollout first. An older artifact without the image field requires
a fresh enabled preview before enabled production can deploy. Static-only
production can accept that empty or missing field.

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
settings and secret selectors while deploying that image. Enable the
preview runtime and complete its rollout to record the rollback digest.
After launch, use a preview
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
