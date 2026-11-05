# Deployment operations

This page gives the work after the [setup](setup.md) of an environment of
the [deployment](README.md): the job images, the database, the
control-plane runtime, the rotations, the zones, and later changes. Run
every command from the repository root, with the shell variables of the
setup. Set `NAMESPACE="ciproxy-$DEPLOY_ENV"` before these commands. The operator
does each step with local credentials and the roles of
[Identities and permissions](identities.md#operator-permissions).

Most steps here use the cluster. Get its credentials first:

```sh
gcloud container clusters get-credentials ciproxy-shared \
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
the main preview host:

```sh
KEY_NAME=ciproxy-main-preview-images-v1
KEY_DIR="$(mktemp -d)"
gcloud secrets versions access "${KEY_NAME##*-v}" \
  --secret="ciproxy-$DEPLOY_ENV-cdn-signing-key" --project="$PROJECT_ID" \
  --out-file="$KEY_DIR/key"
URL="https://br-main.preview.app.ciproxy.dev/images/NAME"
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

Use [database setup](runtime.md#prepare-the-database) after the first deploy.

## Turn on the runtime

Use [runtime setup](runtime.md#turn-on-the-runtime) when a real HTTP image
exists. It gives the image interface, secrets, and rollout steps.

## Rotate the release token

1. Add a new token version, as in
   [step 4 of the setup](setup.md#4-store-the-release-token).
2. Deploy. The deploy copies the new version into the Kubernetes Secret.
3. Restart the release server, because it reads the token only at start:

```sh
kubectl rollout restart "deployment/ciproxy-$DEPLOY_ENV-release-cli-release-server" --namespace="$NAMESPACE"
kubectl rollout status "deployment/ciproxy-$DEPLOY_ENV-release-cli-release-server" --namespace="$NAMESPACE"
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
   `kubectl rollout status deployment/control-plane --namespace="$NAMESPACE"`.
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
kubectl rollout restart deployment/control-plane --namespace="$NAMESPACE"
kubectl rollout status deployment/control-plane --namespace="$NAMESPACE"
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
kubectl get secretproviderclasses --namespace="$NAMESPACE"
kubectl get replicasets --namespace="$NAMESPACE" -l app.kubernetes.io/name=ciproxy-control-plane \
  -o jsonpath='{range .items[*]}{.spec.template.spec.volumes[?(@.name=="secrets")].csi.volumeAttributes.secretProviderClass}{"\n"}{end}'
helm history "ciproxy-$DEPLOY_ENV" --namespace="$NAMESPACE"
helm get manifest "ciproxy-$DEPLOY_ENV" --namespace="$NAMESPACE" --revision=REVISION | grep secretProviderClass:
```

## Change infrastructure or trust

Merge the Terraform change through protected `main`. CI promotes seed and
bootstrap with the central trust identity. It promotes shared infrastructure
with the separate shared identity. The primary preview branch cannot change
trust. After launch, preview reads central infrastructure and production
promotes it after the successful preview of the same commit.

Each environment keeps its own `ALERT_EMAIL`, runtime flag, uptime flag,
and prepared zones in its completed workload artifact. Central edge uses
that exact run/attempt evidence. Snapshot state remains a record only. Changing
production's alert address does not replace
preview's channel. Shared edge waits until workloads and NEGs are ready.

Destructive changes need an exact committed central approval. CI refuses
destructive trust changes even with approval. The operator handles those
changes and lock-out recovery. See [promotion](promotion.md).

After failure, rerun all jobs. Check the writer before releasing a stale
lock. Keep both prelaunch apply accounts disabled after launch.

## Zones and NEGs

The shared zone module holds desired placement and a retained catalog:
`allowed_zones` in
`deploy/terraform/modules/zones/main.tf`. Today it is
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
the zones, edit the list and merge the change. CI applies shared
infrastructure first. Each workload saves the zones that its Helm release
used, only after all corresponding NEGs are ready. Central edge uses that
environment snapshot. A concurrent production apply cannot attach a new
preview zone before preview prepared its NEG.

## Static files and protection

- Site/dashboard origins enforce PAP and remain private. Their public
  content is served through the edge by the viewer-only https-lb agent.
- Every multipart upload stores explicit `cacheControl` object metadata.
  HTML and its route aliases use `no-store`. Assets use
  `public,max-age=86400,must-revalidate`. GCS response headers come from
  this stored metadata, not a request Cache-Control header.
- Static backends keep `USE_ORIGIN_HEADERS`, so HTML no-store remains in
  force. No force-cache policy or new TTL is added. The publisher never
  deletes a file, so open tabs can load old assets.
- Each deploy invalidates the whole CDN cache after the edge apply.
- Terraform cannot delete the GKE cluster, the Cloud SQL instance, the state
  bucket, or the Boat KMS key. Their protection must change first.
- Cloud SQL also has API deletion protection. Secret Manager IDs have
  Terraform provider deletion protection. Secret protection does not stop
  a console deletion or an owner-equivalent IAM change. The planned GCP
  deny policy remains separate work.

Production data and edge have protected prefixes. If a first create finds
an existing production name, stop before populating data. Follow the
[operator state procedure](state.md#unexpected-existing-production-name).
Never import a prelaunch-created object to bypass that failure.

Production finishes its own edge/invalidation before promoting preview's
separate edge. New tag bindings use permanent numeric map IDs. Tag/IAM
propagation can delay invalidation rights. Rerun all jobs after propagation
rather than changing the tag with gcloud or reusing a saved binary.

Namespace policy labels remain operator-owned. After a namespace setup
change, reapply both manifests with `namespace_access.py` before untrusted
deploys resume. Baseline enforces/audits/warns on each namespace. It rejects
host filesystem mounts without widening deployment roles.

DNS authority stays split as [DNS setup](dns.md) describes. Cloudflare keeps
`ciproxy.dev` and Workspace mail. Google owns only production app/get parents
and their preview child delegations. The manual DNS-only apex A record
uses `terraform/edge`'s production backend `public_ip` output.

A central scope with no roots is a successful promotion, not a skipped apply
job. Its read-only status check needs no apply identity or plan artifact.
If private-origin IAM is refused at first live setup, stop and follow
[the item 76 fallback](identities.md#private-static-origins). Keep the
organization rule unchanged.
