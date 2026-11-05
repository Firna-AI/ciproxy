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
no schema rights. No migration exists in this repository yet. Set the
`postgres` password at a prompt, and start a proxy pod with the runtime
identity:

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
`kubectl delete pod ciproxy-sql-admin --namespace="$NAMESPACE"`. Run the
migrations as the runtime user, so that it owns its tables.

## Turn on the runtime

Do this only when a real control-plane image exists. The current binary
does not meet this interface of the `deploy/helm/ciproxy` chart:

| Part | What the image must do |
| --- | --- |
| HTTP | Listen on `HTTP_BIND`, which is `0.0.0.0:8080`. |
| Health | Answer `GET /api/health/ready` when ready, and `GET /api/health/live` while alive. The load balancer checks the ready path with the dashboard host. |
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

1. Push the image to the shared repository `ciproxy-shared-runtime`. The GKE nodes
   pull only from there.
2. Add a version to each of the six other secrets `ciproxy-<env>-<file>` of
   the table, from a private file. [Key registration](setup.md#9-register-the-cdn-signing-key)
   already gave the CDN key its numbered version. The pod cannot mount a
   secret without a version. For
   example:

   ```sh
   gcloud secrets versions add "ciproxy-$DEPLOY_ENV-session-key" \
     --project="$PROJECT_ID" --data-file=PRIVATE_FILE
   ```

3. [Prepare the database](#prepare-the-database), and run the migrations.
4. Check that `CDN_SIGNING_KEY_NAME` holds the name from key registration. Set
   `CONTROL_PLANE_IMAGE` and `CONTROL_PLANE_RUNTIME_READY=true` in the
   GitHub environment. Then deploy, and check the rollout with
   `kubectl rollout status deployment/control-plane --namespace="$NAMESPACE"`.

The deploy runs one replica with the proxy beside it, and adds its endpoints
to the edge. With uptime checks on, it also checks `/api/health/ready`. To
turn off a running runtime, set `CONTROL_PLANE_RUNTIME_READY` to `false`
and deploy. While a runtime runs, an unset value stops the deploy with an
error, so a lost variable cannot remove the runtime.
