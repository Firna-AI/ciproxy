# Control-plane runtime setup

Run these operator steps after [deployment setup](setup.md). Set
`PROJECT_ID=ciproxy`, `DEPLOY_ENV`, and `NAMESPACE="ciproxy-$DEPLOY_ENV"`.
Use the [operator permissions](identities.md#operator-permissions).
Complete [UID setup](runtime-identity.md) before using the runtime identity.
Its Google binding trusts the real operator-created ServiceAccount UID,
not a namespace/name that another cluster can reuse. A recreated account
needs the documented UID recovery before the runtime can authenticate.

## Prepare the database

Do this only after the environment's protected central data apply passed,
and before the runtime turns on. Do not prepare a database behind a failed
production name collision. See [state recovery](state.md). Terraform
creates the IAM database user `ciproxy-<env>-control@<project>.iam`, with
no schema rights. Set the `postgres` password at a prompt, and start a
proxy pod with the runtime identity:

```sh
gcloud sql users set-password postgres --instance="ciproxy-$DEPLOY_ENV" \
  --project="$PROJECT_ID" --prompt-for-password
kubectl run ciproxy-sql-admin --namespace="$NAMESPACE" \
  --image=gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.27.0 \
  --overrides='{"apiVersion":"v1","spec":{"serviceAccountName":"control-plane"}}' \
  -- --private-ip --port=5432 "$PROJECT_ID:us-central1:ciproxy-$DEPLOY_ENV"
kubectl wait --namespace="$NAMESPACE" --for=condition=Ready pod/ciproxy-sql-admin --timeout=10m
kubectl port-forward --namespace="$NAMESPACE" pod/ciproxy-sql-admin 15432:5432
```

In a second terminal, connect with
`psql -h 127.0.0.1 -p 15432 -U postgres -d ciproxy -W`, and grant the
rights to the `sql_user` of the deployment output. For the main preview:

```sql
GRANT CONNECT ON DATABASE ciproxy TO "ciproxy-main-preview-control@ciproxy.iam";
GRANT USAGE, CREATE ON SCHEMA public TO "ciproxy-main-preview-control@ciproxy.iam";
```

Stop the port forward, and run
`kubectl delete pod ciproxy-sql-admin --namespace="$NAMESPACE"`. At startup,
the control plane applies embedded migrations as the runtime database user.
The existing schema rights let that user create and own its tables.

## Prepare the session key

The mounted `session-key` file contains base64url text that encodes exactly
32 raw random bytes. The runtime accepts this text with or without padding.
Generate it into a file in an existing private directory:

```sh
umask 077
openssl rand -base64 32 | tr '+/' '-_' > /private/session-key
```

Keep the file private. Upload it to the existing session-key secret in the
runtime steps below. The CDN key keeps its separate 16-byte format.

## Turn on the runtime

Complete [automatic image setup](control-plane-images.md) before enabling
the runtime. Preview builds or reuses its image on each accepted push.
Production promotes the same saved digest. The `deploy/helm/ciproxy` chart
supplies this interface:

| Part              | What the image must do                                                                                                                                                                                                                                                                                                                                                                        |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| HTTP              | Listen on `HTTP_BIND`, which is `0.0.0.0:8080`.                                                                                                                                                                                                                                                                                                                                               |
| Health            | Answer `GET /api/health/ready` when ready, and `GET /api/health/live` while alive. The load balancer checks the ready path with the dashboard host.                                                                                                                                                                                                                                           |
| Database          | Use `DATABASE_URL`: the Cloud SQL Auth Proxy beside it, on `127.0.0.1:5432`, with IAM login and no password.                                                                                                                                                                                                                                                                                  |
| Settings          | Read `PUBLIC_URL`, `IMAGE_URL_PREFIX`, `BOAT_KMS_KEY`, and `CDN_SIGNING_KEY_NAME`. The chart sets `CDN_SIGNING_KEY_NAME` to `ciproxy-<env>-images-v<N>`.                                                                                                                                                                                                                                      |
| Boat snapshot     | Read the optional `BOAT_SHARED_SNAPSHOT` pod variable from a Kubernetes Secret. See [Boat setup](#boat-shared-snapshot).                                                                                                                                                                                                                                                                      |
| GitHub App        | Read the pod variables `GITHUB_APP_ID`, `GITHUB_OAUTH_CLIENT_ID`, and `GITHUB_APP_SLUG`. Deploy maps them from this environment's `CIPROXY_APP_ID`, `CIPROXY_APP_CLIENT_ID`, and `CIPROXY_APP_SLUG` GitHub variables. Build the installation URL from the slug. The OAuth secret stays in `github-oauth-secret`.                                                                              |
| Job image         | Read `JOB_IMAGE_MANIFEST_PATH` from this environment's GitHub variable. It pins one immutable path relative to `IMAGE_URL_PREFIX`, such as `<release>/manifest.json`.                                                                                                                                                                                                                         |
| Build and install | Read `BUILD_ID` from the trusted deployment source commit. Read `DOWNLOAD_URL` from the HTTPS origin of this environment's release host. Deploy derives both values.                                                                                                                                                                                                                          |
| Secrets           | Read the files `github-app-private-key`, `github-webhook-secret`, `github-oauth-secret`, `stripe-secret-key`, `stripe-webhook-secret`, `session-key`, and `cdn-signing-key` in `SECRETS_DIRECTORY`, which is `/var/run/secrets/ciproxy`. The `cdn-signing-key` file holds secret version N, which `CDN_SIGNING_KEY_NAME` names. Each other file holds the latest version when the pod starts. |
| Process           | Run as a non-root user with a read-only root file system, and stop within 300 seconds.                                                                                                                                                                                                                                                                                                        |

The runtime signs and fetches the manifest and its sibling `downloads.json`
with the existing CDN key. It reads `root_disk_bytes` and `image_id` only
from the manifest. It reads artifact names, hashes, and download bytes only
from `downloads.json`. It signs fresh artifact URLs for each send.

The GitHub variable `CDN_SIGNING_KEY_NAME` is the one selector of the CDN
key. From it, the chart sets both the key name and the secret version N of
each pod. A value that is missing, malformed, or of the other environment
selects version 0. That version cannot exist, so the pod cannot mount its
secrets and does not start. Helm then waits ten minutes, and the deploy
fails. The chart has no earlier check. Old pods keep running, because the
rollout keeps each old pod until a new pod is ready. While the runtime is
off, the chart renders no control plane, so the selector has no effect.

Then do these steps:

1. Confirm that preview published or reused its image in
   `ciproxy-shared-runtime`. The GKE nodes pull only from there.
2. Add a version to each of the six other secrets `ciproxy-<env>-<file>` of
   the table, from a private file.
   [Key registration](setup.md#9-register-the-cdn-signing-key) already gave
   the CDN key its numbered version. The pod cannot mount a
   secret without a version. For
   example:

   ```sh
   gcloud secrets versions add "ciproxy-$DEPLOY_ENV-session-key" \
     --project="$PROJECT_ID" --data-file=PRIVATE_FILE
   ```

3. [Prepare the database](#prepare-the-database) for startup migrations.
4. Check that `CDN_SIGNING_KEY_NAME` holds the name from key registration. Set
   the [runtime variables](setup.md#4-set-github-variables), including
   `CONTROL_PLANE_RUNTIME_READY=true`, in the
   GitHub environment. Then deploy, and check the rollout with
   `kubectl rollout status deployment/control-plane --namespace="$NAMESPACE"`.

The deploy runs one replica with the proxy beside it, and adds its endpoints
to the edge. With uptime checks on, it also checks `/api/health/ready`. To
turn off a running runtime, set `CONTROL_PLANE_RUNTIME_READY` to `false`
and deploy. While a runtime runs, an unset value stops the deploy with an
error, so a lost variable cannot remove the runtime.

## Boat shared snapshot

`BOAT_SHARED_SNAPSHOT` selects the shared Boat snapshot of the job image.
An absent or empty value disables Boat. Own machines still run jobs.
Use the snapshot of the same image release as `JOB_IMAGE_MANIFEST_PATH`.
Set both before the next deploy. The runtime does not compare them.

The `deploy/terraform/data` root creates one Secret Manager parent per
environment, with no version. Apply it through the normal CI promotion for
both `main-preview/data` and `production/data` state prefixes. No bootstrap,
shared, edge, or environment snapshot root needs an apply for this change.
Each `ciproxy-<env>-deploy` identity gets
`roles/secretmanager.secretAccessor` on its own
`ciproxy-<env>-boat-shared-snapshot` parent, as for its release token.

After that data apply succeeds, run these commands as the operator.
Repeat them for `DEPLOY_ENV=production` after its data apply succeeds.
The prompt keeps the share code out of shell arguments, files, and logs:

```sh
export DEPLOY_ENV=main-preview
gcloud secrets describe "ciproxy-$DEPLOY_ENV-boat-shared-snapshot" \
  --project=ciproxy --format='value(name)'
python3 - <<'PY'
import getpass
import os
import subprocess

subprocess.run([
    "gcloud", "secrets", "versions", "add",
    f"ciproxy-{os.environ['DEPLOY_ENV']}-boat-shared-snapshot",
    "--project=ciproxy", "--data-file=-",
], input=getpass.getpass("Boat share code: "), text=True, check=True)
PY
```

Set the existing `JOB_IMAGE_MANIFEST_PATH` GitHub environment variable to
the matching immutable release path. Run a fresh accepted deploy of this
code, with a control-plane image that supports Boat. Use the
[all-jobs retry](first-deploy.md#retry-all-jobs-and-create-the-edge) when
retrying a selected run. Add no GitHub variable or secret for the share code.

Deploy reads `latest` into memory and applies `boat-shared-snapshot` in
the environment namespace over standard input. It uses server-side apply,
so no last-applied annotation stores the value. Helm receives only the
Kubernetes Secret revision. The pod checksum hashes that public revision,
not the share code. A changed revision restarts the control-plane pods.
A new Secret Manager version reaches the pods at the next deploy.

Without a version, deploy applies an empty Secret and continues. It removes
any previous `BOAT_SHARED_SNAPSHOT` key that it owns. The chart's optional
key reference then leaves the variable unset. Access denials, disabled
versions, and apply failures stop the deploy with no payload in the log.
Keep a usable version while Boat sandboxes can exist. Without the value,
the runtime stops Boat cleanup and removes the exit callback.

`BOAT_KMS_KEY` already uses the full resource path:

```text
projects/ciproxy/locations/us-central1/keyRings/ciproxy-<env>/cryptoKeys/boat-account-keys
```

The runtime identity `ciproxy-<env>-control` already has
`roles/cloudkms.cryptoKeyEncrypterDecrypter` on that environment's key.
This role supplies `cloudkms.cryptoKeyVersions.useToEncrypt` and
`cloudkms.cryptoKeyVersions.useToDecrypt`. No KMS role changes are needed.
The runtime gets access tokens from the GKE metadata service through its
[UID binding](runtime-identity.md). Offline checks cannot prove live IAM
access or a real Boat start.
