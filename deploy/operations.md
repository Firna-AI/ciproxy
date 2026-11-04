# Deployment operations

This page gives the work after the [setup](setup.md) of an environment of
the [deployment](README.md): the job images, the database, the
control-plane runtime, the rotations, the zones, and later changes. Run
every command from the repository root, with the shell variables of the
setup. The operator does each step with local credentials and the roles of
[Identities and permissions](setup.md#identities-and-permissions).

Most steps here use the cluster. Get its credentials first:

```sh
gcloud container clusters get-credentials "ciproxy-$DEPLOY_ENV" \
  --region=europe-west1 --project="$PROJECT_ID" --dns-endpoint
```

## Publish job images

No code in this repository builds or publishes the job images yet. Publish
each image to the private bucket `ciproxy-<env>-images-<project>` after
steps 8 and 9 of the setup. The deploy identity cannot write this bucket.
Use the operator account, or grant a separate publisher
`roles/storage.objectCreator`, which can add objects but not replace or
delete them. Put each file under `images/` with a new name.

```sh
IMAGE_BUCKET="ciproxy-$DEPLOY_ENV-images-$PROJECT_ID"
gcloud storage buckets add-iam-policy-binding "gs://$IMAGE_BUCKET" \
  --member=PUBLISHER --role=roles/storage.objectCreator
gcloud storage cp FILE "gs://$IMAGE_BUCKET/images/NAME"
```

Then check the signed access. Set `KEY_NAME` to the current value of
`CDN_SIGNING_KEY_NAME`. The probe reads the exact secret version N that this
name selects, never `latest`, into a private directory. The example uses
the staging host:

```sh
KEY_NAME=ciproxy-staging-images-v1
KEY_DIR="$(mktemp -d)"
gcloud secrets versions access "${KEY_NAME##*-v}" \
  --secret="ciproxy-$DEPLOY_ENV-cdn-signing-key" --project="$PROJECT_ID" \
  --out-file="$KEY_DIR/key"
URL="https://app.staging.ciproxy.dev/images/NAME"
SIGNED="$(gcloud compute sign-url "$URL" --key-name="$KEY_NAME" --key-file="$KEY_DIR/key" \
  --expires-in=10m --format='value(signedUrl)')"
EXPIRED="$(gcloud compute sign-url "$URL" --key-name="$KEY_NAME" --key-file="$KEY_DIR/key" \
  --expires-in=1s --format='value(signedUrl)')"
rm -rf "$KEY_DIR"
sleep 10
for request in "$SIGNED" "${SIGNED}0" "$EXPIRED" "$URL" \
  "https://storage.googleapis.com/$IMAGE_BUCKET/images/NAME"; do
  curl -s -o /dev/null -w '%{http_code}\n' "$request"
done
```

Only the first request, with the valid signature, may return 200. The
changed signature, the expired URL, the unsigned URL, and the direct Cloud
Storage URL must each fail.

## Prepare the database

Do this after the first deploy and before the runtime turns on. Terraform
creates the IAM database user `ciproxy-<env>-control@<project>.iam`, with
no schema rights. No migration exists in this repository yet. Set the
`postgres` password at a prompt, and start a proxy pod with the runtime
identity:

```sh
gcloud sql users set-password postgres --instance="ciproxy-$DEPLOY_ENV" \
  --project="$PROJECT_ID" --prompt-for-password
kubectl run ciproxy-sql-admin --namespace=ciproxy \
  --image=gcr.io/cloud-sql-connectors/cloud-sql-proxy:2.27.0 \
  --overrides='{"apiVersion":"v1","spec":{"serviceAccountName":"control-plane"}}' \
  -- --private-ip --port=5432 "$PROJECT_ID:europe-west1:ciproxy-$DEPLOY_ENV"
kubectl wait --namespace=ciproxy --for=condition=Ready pod/ciproxy-sql-admin --timeout=10m
kubectl port-forward --namespace=ciproxy pod/ciproxy-sql-admin 15432:5432
```

In a second terminal, connect with
`psql -h 127.0.0.1 -p 15432 -U postgres -d ciproxy -W`, and grant the
rights to the `sql_user` of the deployment output. For staging:

```sql
GRANT CONNECT ON DATABASE ciproxy TO "ciproxy-staging-control@internal-498318.iam";
GRANT USAGE, CREATE ON SCHEMA public TO "ciproxy-staging-control@internal-498318.iam";
```

Stop the port forward, and run
`kubectl delete pod ciproxy-sql-admin --namespace=ciproxy`. Run the
migrations as the runtime user, so that it owns its tables.

## Turn on the runtime

Do this only when a real control-plane image exists. The current binary
does not meet this interface of the `deploy/helm/ciproxy` chart:

| Part | What the image must do |
| --- | --- |
| HTTP | Listen on `HTTP_BIND`, which is `0.0.0.0:8080`. |
| Health | Answer `GET /api/health/ready` when ready, and `GET /api/health/live` while alive. The load balancer checks the ready path with the `app.` host. |
| Database | Use `DATABASE_URL`: the Cloud SQL Auth Proxy beside it, on `127.0.0.1:5432`, with IAM login and no password. |
| Settings | Read `PUBLIC_URL`, `IMAGE_URL_PREFIX`, `BOAT_KMS_KEY`, and `CDN_SIGNING_KEY_NAME`. The chart sets `CDN_SIGNING_KEY_NAME` to `ciproxy-<env>-images-v<N>`. |
| Secrets | Read the files `github-app-private-key`, `github-webhook-secret`, `github-oauth-secret`, `stripe-secret-key`, `stripe-webhook-secret`, `session-key`, and `cdn-signing-key` in `SECRETS_DIRECTORY`, which is `/var/run/secrets/ciproxy`. The `cdn-signing-key` file holds secret version N, which `CDN_SIGNING_KEY_NAME` names. Each other file holds the latest version when the pod starts. |
| Process | Run as a non-root user with a read-only root file system, and stop within 300 seconds. |

The GitHub variable `CDN_SIGNING_KEY_NAME` is the one selector of the CDN
key. From it, the chart sets both the key name and the secret version N of
each pod. A value that is missing, malformed, or of the other environment
selects version 0. That version cannot exist, so the pod cannot mount its
secrets and does not start. Helm then waits ten minutes, and the deploy
fails. The chart has no earlier check. Old pods keep running, because the
rollout keeps each old pod until a new pod is ready. While the runtime is
off, the chart renders no control plane, so the selector has no effect.

Then do these steps:

1. Push the image to the repository `ciproxy-<env>-runtime`. The GKE nodes
   pull only from there.
2. Add a version to each of the six other secrets `ciproxy-<env>-<file>` of
   the table, from a private file. Setup step 8 already gave the CDN key its
   numbered version. The pod cannot mount a secret without a version. For
   example:

   ```sh
   gcloud secrets versions add "ciproxy-$DEPLOY_ENV-session-key" \
     --project="$PROJECT_ID" --data-file=PRIVATE_FILE
   ```

3. [Prepare the database](#prepare-the-database), and run the migrations.
4. Check that `CDN_SIGNING_KEY_NAME` holds the name of setup step 8. Set
   `CONTROL_PLANE_IMAGE` and `CONTROL_PLANE_RUNTIME_READY=true` in the
   GitHub environment. Then deploy, and check the rollout with
   `kubectl rollout status deployment/control-plane --namespace=ciproxy`.

The deploy runs one replica with the proxy beside it, and adds its endpoints
to the edge. With uptime checks on, it also checks `/api/health/ready`. To
turn off a running runtime, set `CONTROL_PLANE_RUNTIME_READY` to `false`
and deploy. While a runtime runs, an unset value stops the deploy with an
error, so a lost variable cannot remove the runtime.

## Rotate the release token

1. Add a new token version, as in
   [step 4 of the setup](setup.md#4-store-the-release-token).
2. Deploy. The deploy copies the new version into the Kubernetes Secret.
3. Restart the release server, because it reads the token only at start:

```sh
kubectl rollout restart deployment/cli-release-server --namespace=ciproxy
kubectl rollout status deployment/cli-release-server --namespace=ciproxy
```

Then revoke the old token on GitHub.

## Rotate the CDN signing key

1. Register a new key, as in
   [step 8 of the setup](setup.md#8-register-the-cdn-signing-key). The
   script adds the next secret version N and prints
   `ciproxy-<env>-images-v<N>`. The old key stays valid on the backend
   bucket.
2. Set `CDN_SIGNING_KEY_NAME` to the printed name.
3. Deploy. The new name gives the pods a new key name and a new provider
   object, so the rollout replaces each pod. Each new pod mounts version N
   and gets its name together. Each old pod keeps the old pair until it
   stops.
4. When the runtime runs, confirm its rollout with
   `kubectl rollout status deployment/control-plane --namespace=ciproxy`.
5. Wait until each URL that the old key signed has expired.
6. Delete the old key name from the backend bucket:

```sh
gcloud compute backend-buckets delete-signed-url-key "ciproxy-$DEPLOY_ENV-images" \
  --key-name=OLD_KEY_NAME --project="$PROJECT_ID"
```

After that, do not roll the control plane back to a Helm revision or a
ReplicaSet that uses the old name, because its URLs would fail. To run an
older application image, set `CONTROL_PLANE_IMAGE` to that image, keep the
current `CDN_SIGNING_KEY_NAME`, and deploy.

## Rotate a runtime secret

The Secret Manager add-on of the cluster has automatic rotation off. So the
secret files of a pod keep the versions that it mounted when it started. A
deploy that changes the image or an environment variable of the control
plane starts new pods. Otherwise, after you add a new version of a runtime
secret from a private file, restart the control plane:

```sh
kubectl rollout restart deployment/control-plane --namespace=ciproxy
kubectl rollout status deployment/control-plane --namespace=ciproxy
```

This rule does not apply to the CDN key. Its pods mount the exact version
that `CDN_SIGNING_KEY_NAME` names, so use the CDN key rotation instead.

## Keep the old provider objects

With the runtime on, a `SecretProviderClass` lists the secret files of the
pods. Its name is `control-plane-` and a hash of its full file list, which
includes the CDN key version. The pod template names that exact object. So
a new CDN key or a new secret file gives a new object, and replaces the
pods.

Helm keeps each older object, because of the annotation
`helm.sh/resource-policy: keep`, and then no longer manages it. So old pods,
old ReplicaSets, and a rollback to a kept Helm revision can still mount
their files. These objects hold secret names and version numbers, not
secret values.

Do not change or delete an old object while a pod, a ReplicaSet, or a Helm
revision can still use it. Kubernetes keeps up to 10 old ReplicaSets of the
control plane, and the deploy keeps 5 Helm revisions. Before you delete an
object, check that no ReplicaSet and no kept Helm revision names it:

```sh
kubectl get secretproviderclasses --namespace=ciproxy
kubectl get replicasets --namespace=ciproxy -l app.kubernetes.io/name=ciproxy-control-plane \
  -o jsonpath='{range .items[*]}{.spec.template.spec.volumes[?(@.name=="secrets")].csi.volumeAttributes.secretProviderClass}{"\n"}{end}'
helm history ciproxy --namespace=ciproxy
helm get manifest ciproxy --namespace=ciproxy --revision=REVISION | grep secretProviderClass:
```

## Change the foundation or the bootstrap

The deploy workflows apply only the edge root. Apply a change to the
bootstrap or foundation roots locally, with the commands of
[step 2 of the setup](setup.md#2-bootstrap-the-environment).
Read each plan before you confirm it. Then deploy, so that the edge reads
the new foundation outputs.

## Zones and NEGs

The foundation holds one zone list: `allowed_zones` in
`deploy/terraform/modules/foundation/variables.tf`. Today it is
`europe-west1-b`, `europe-west1-c`, and `europe-west1-d`. Four parts use
this list:

- The release server and the control plane have a required node affinity
  to these zones.
- The `zones` field of the `cloud.google.com/neg` annotation of each Service
  makes GKE create the NEGs in all these zones before a pod runs there.
- The deploy waits until the NEG status of each Service lists every zone.
- The edge attaches the NEG of each zone to its backend service.

The `zones` field is a GKE Preview feature. Google can change a Preview
feature, and gives it no SLA. It needs GKE 1.36.2-gke.3104000 or later. The
cluster sets the minimum version 1.36.4 and stays on the Regular release
channel. Keep the cluster at a version that supports the feature. To change
the zones, edit the list and apply the foundation locally. Then deploy, so
that the workloads, the NEGs, and the edge get the new list.

## Static files and protection

- HTML files have `no-store`, and other files a one-day cache. The
  publisher never deletes a file, so open tabs can load their old files.
- Each deploy invalidates the whole CDN cache after the edge apply.
- Terraform cannot delete the GKE cluster, the Cloud SQL instance, the state
  bucket, or the Boat KMS key. Their protection must change first.
