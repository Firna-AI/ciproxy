# Build the control-plane container

The deploy workflow builds and publishes this image automatically.
It reuses unchanged source inputs. An enabled preview deploys an immutable
digest and records it after readiness. Enabled production promotes that
deployed digest from the exact saved preview workload. A disabled preview
records no deployed image. Static-only production can still proceed.
[Image deployment](../../deploy/control-plane-images.md) gives identity
setup, source reuse, and explicit rollback. The commands below support
local builds and operator recovery.

The image runs the hosted Rust control plane. Build it from the repository
root. Docker builds the release executable inside the Rust 1.99.1 builder.
The Debian Bookworm runtime has PostgreSQL client libraries and CA roots.
The image runs as UID and GID 10001. It starts the executable with no
arguments, so the container serves.

The [crate guide](README.md) gives the runtime settings. The
[deployment interface](../../deploy/runtime.md#turn-on-the-runtime) gives
the Helm requirements. Set all required settings before starting the
service. Mount its secrets as files. Do not pass secrets to the build.

## Build

Install Docker and Bash. A build needs no registry credentials. All Cargo
dependencies are public. The root `.dockerignore` limits the context to the
Rust workspace. It excludes private operator files and local build output.
CI also selects tracked source files, as the
[image deployment guide](../../deploy/control-plane-images.md) describes.

```sh
bash crates/ciproxy-control-plane/scripts/container-image.sh \
  ciproxy-control-plane:local
```

The command ends with `Built ciproxy-control-plane:local`. It leaves the
image in the selected Docker engine. It does not start or deploy it.
Use `--network host` when the builder needs the host network. Use
`--platform linux/amd64` when the target uses that platform. The script
passes these options to `docker build`. Use the normal `DOCKER_HOST`,
`DOCKER_TLS_VERIFY`, and `DOCKER_CERT_PATH` settings for another engine.

To record a known source revision in the image label, set
`CIPROXY_BUILD_REVISION`. The default label is `unknown`. Build only a clean
checkout when the label must identify the exact source.

```sh
CIPROXY_BUILD_REVISION=$(git rev-parse HEAD) \
  bash crates/ciproxy-control-plane/scripts/container-image.sh \
  ciproxy-control-plane:local
docker image inspect ciproxy-control-plane:local \
  --format '{{.Config.User}} {{.Config.StopSignal}}'
```

The inspection prints `10001:10001 SIGTERM`.

## Push

The deployment pulls from this shared Artifact Registry repository:

```text
us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime
```

Prepare a private Docker config JSON file outside the repository. Use
temporary credentials with write access to that repository. This is its
schema. `auth` holds the base64 encoding of `USERNAME:PASSWORD_OR_TOKEN`.
Base64 does not encrypt the value.

```json
{
  "auths": {
    "us-central1-docker.pkg.dev": {
      "auth": "BASE64_USERNAME_COLON_PASSWORD_OR_TOKEN"
    }
  }
}
```

For an Artifact Registry OAuth access token, the username is
`oauth2accesstoken`. Prepare the file with private tools. Do not put the
token in shell history, command arguments, or the build context. Keep the
file readable only by its owner. The script uses inline `auths` entries.
Do not use a credential helper in this file.

```sh
export CIPROXY_REGISTRY_AUTH_FILE=/private/path/registry-config.json
chmod 600 "$CIPROXY_REGISTRY_AUTH_FILE"
export CIPROXY_CONTROL_PLANE_IMAGE=\
us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime/control-plane:VERSION
bash crates/ciproxy-control-plane/scripts/container-image.sh \
  "$CIPROXY_CONTROL_PLANE_IMAGE" --push --platform linux/amd64
```

Replace `VERSION` with the release tag. The script builds the image first.
It then copies the config into a temporary private directory for the push.
It removes that directory on success, failure, or a handled signal. It
keeps the original credentials file. Remove that file when it expires.

Docker prints `digest: sha256:...` after a successful push. The script then
prints `Pushed IMAGE. Use its registry digest for deployment.` For operator
recovery, select the repository path followed by `@sha256:DIGEST` with the
[image rollback](../../deploy/control-plane-images.md#roll-back-and-resume).
Remove the tag from that path. Normal deployments select their image
automatically. Follow the separate
[runtime setup](../../deploy/runtime.md) before deployment. A successful
build or push does not enable the runtime.

## Run and check

Use the built image in the selected Docker engine. The smoke script does
not pull an image. Install curl for its HTTP checks. Prepare a private
environment file with the settings from the crate guide.
Prepare a private directory with the required secret files. UID 10001 must
have read access to the secret files and search access to their directories.
Do not copy either input into the image.

The service fetches the pinned image release before it opens its HTTP port.
The container must reach `IMAGE_URL_PREFIX`. `JOB_IMAGE_MANIFEST_PATH` must
name a published manifest. The `cdn-signing-key` file must hold the key
that `CDN_SIGNING_KEY_NAME` names. The service then connects to the
database. Readiness follows its first reconciliation pass. A failed release
fetch or database connection stops the container before readiness.

```sh
export CIPROXY_RUNTIME_ENV_FILE=/private/path/control-plane.env
export CIPROXY_RUNTIME_SECRETS_DIR=/private/path/control-plane-secrets
bash crates/ciproxy-control-plane/scripts/container-smoke.sh \
  ciproxy-control-plane:local
```

The script uses a unique container name and a read-only root file system.
It waits for readiness, then checks liveness. It gives SIGTERM 300 seconds
to stop the process. It requires exit code `0` and `OOMKilled=false`.
It removes its container on success, failure, or a handled signal.
It does not print application logs or private input contents. Success prints
`Container smoke passed: ready, live, exit 0, OOMKilled false.`

The default network publishes a random port on `127.0.0.1`. The database
address must be reachable inside the container. For a database on the host,
add `--network host` on Linux. That mode checks `127.0.0.1:8080`.
Use `--add-host HOST:ADDRESS` for a test-only host mapping. Set `DATABASE_URL`
in the private environment file to the isolated smoke database.
