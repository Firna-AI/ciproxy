# Deployment setup

Set up main preview first. Use the [launch guide](launch.md) for production.
Both environments use project `ciproxy`. Run commands from the repository
root. No deployment uses `internal-498318`.

```sh
DEPLOY_ENV=main-preview
PROJECT_ID=ciproxy
NAMESPACE="ciproxy-$DEPLOY_ENV"
```

## 1. Prepare the project

Check billing and organization policies. The futex.ai domain-restricted
sharing rule stays in force. Site/dashboard origins are private with public
access prevention enforced. No public grant or organization-policy
exception is needed. The edge grants its Google-created service agent
viewer access after backend creation. Read the operator roles in
[Deployment identities](identities.md#operator-permissions).

```sh
gcloud billing projects describe "$PROJECT_ID"
gcloud auth login
gcloud auth application-default login
gcloud auth application-default set-quota-project "$PROJECT_ID"
```

Install Terraform 1.17.1, Helm 4.3.1, Google Cloud CLI, kubectl,
`gke-gcloud-auth-plugin`, Docker, GitHub CLI, Python 3, jq, and psql. CI uses
Google Cloud CLI 592.0.0. The pinned Helm installer verifies the archive
checksum and supports Linux/macOS on amd64/arm64.

```sh
bash deploy/scripts/install-helm.sh
export PATH="$PWD/.context/tools/ciproxy-helm:$PATH"
gh auth login
```

A zone on the last free Cloud DNS name-server set can need a TXT ownership
check. Complete that check before retrying. See
[DNS limits](https://docs.cloud.google.com/dns/quotas#name_server_limits).

## 2. Bootstrap

Keep `PRODUCTION_DEPLOY_ENABLED=false` as a repository variable. Create the
initial seed and trust with the operator account:

```sh
gh variable set PRODUCTION_DEPLOY_ENABLED --repo=Firna-AI/ciproxy --body=false
bash deploy/scripts/bootstrap.sh main-preview
```

Seed creates four versioned state buckets and enables trust APIs. The
script uses a temporary local backend until the state bucket exists. It
then asks Terraform to migrate seed state to `trust/seed`. Review both
locations. It never forces an old local copy over remote state.

Bootstrap prepares all nine remote backends with operator credentials.
Plan identities cannot create state. It creates both environments' CI and
runtime identities, plus central identities. It creates no production
SQL, secrets, application buckets, zones, or workloads. Back up
`.context/deploy/ciproxy/seed.tfstate` outside Git. On another operator
machine, use `bootstrap.sh main-preview --existing-state` to read existing
state without another local seed apply.

Set the twelve identity variables from step 6 now. Start a preview push
run. It can create central infrastructure before it stops at a missing
release image digest. On the primary branch, it reads seed/bootstrap and
cannot change trust. Initial operator bootstrap supplies that trust.
Future trust changes promote automatically on protected `main`.

After the cluster exists, install the namespace boundary as the operator.
First grant the operator GKE setup access as the identity guide explains.
Then run:

```sh
python3 deploy/scripts/namespace_access.py main-preview
```

The shared root installs runtime federation after GKE creates its workload
identity pool. Prelaunch shared apply can change only the preview runtime
account's policy. Production shared apply can maintain both runtime
accounts. Neither receives project IAM administration. No extra operator
IAM apply is needed after cluster creation.
[GKE setup order](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/workload-identity)

Both Namespace manifests enforce, audit, and warn at Baseline. Only the
operator manages these labels. Existing namespaces must be reapplied before
untrusted deploys resume. Follow the exact [admission procedure](identities.md#baseline-admission).

The namespace command also creates its namespace-only RoleBinding and
network policy. It permits same-namespace traffic and blocks other
workload namespaces. CI cannot install or change RBAC. Shared apply has
cluster lifecycle permissions and no Kubernetes object verbs.

## 3. Push the release image

The shared repository is `ciproxy-shared-runtime`. Docker must run locally.
The script builds the pinned upstream source, runs its help command,
pushes the image, and prints `RELEASE_SERVER_IMAGE_DIGEST`.

```sh
bash deploy/scripts/mirror-release-image.sh "$DEPLOY_ENV"
```

The operator GitHub account needs read access to `futex-ai/cli-release`.
Optional `published` mode mirrors its published image and needs
`read:packages`. Both environments can use the same immutable digest.

## 4. Store the release token

Create a fine-grained GitHub token with read-only access to public
repositories only. `Firna-AI/ciproxy` and its release assets are public, as
item 70 says. The release server requires a token for the GitHub API when
its repository is configured. It needs no private repository access.
Enter the token at a hidden prompt. Each environment has its own Secret
Manager secret.

```sh
read -rs RELEASE_TOKEN
printf '%s' "$RELEASE_TOKEN" | gcloud secrets versions add \
  "ciproxy-$DEPLOY_ENV-release-github-token" --project=ciproxy --data-file=-
unset RELEASE_TOKEN
```

Deploy copies it through standard input to
`ciproxy-<env>/release-github-token`. Terraform never reads its value.

## 5. Delegate DNS

Keep `ciproxy.dev`, Workspace mail, registrar NS, DNSSEC settings, and
existing preview NS sets on Cloudflare. Preview retains its three zones.
Production creates only app/get zones. Apply their preview child NS records
in Google before delegating the parents from Cloudflare.

| Type in Cloudflare | Name | Value |
| --- | --- | --- |
| NS | `app` | Four name servers of `ciproxy-production-dashboard` |
| NS | `get` | Four name servers of `ciproxy-production-releases` |
| A, **DNS only** | `@` | The production edge `public_ip` output |

Add the apex A record once after production edge applies. Google continues
to serve its certificate/CDN. [DNS setup](dns.md) gives the exact backend
init/output commands, required child delegations, and operator order.
There is no Cloudflare Terraform provider or CI credential.

## 6. Set GitHub variables

Create GitHub environment `main-preview`. Read its twelve identity values:

```sh
gh api --method PUT "repos/Firna-AI/ciproxy/environments/$DEPLOY_ENV"
terraform -chdir=deploy/terraform/bootstrap output -json github_environment_variables \
  | jq '.["main-preview"]'
gh variable set GCP_WIF_PROVIDER --repo=Firna-AI/ciproxy \
  --env="$DEPLOY_ENV" --body=VALUE
```

Set every printed identity variable in that GitHub environment. They cover
workload, environment plan/apply, central plan/apply, and central trust
apply. Production later uses its own printed map. Never copy preview's
central apply identities into production.

| Variable | Value |
| --- | --- |
| `ALERT_EMAIL` | This environment's alert address. It stays separate from the other environment. |
| `RELEASE_SERVER_IMAGE_DIGEST` | The `sha256:` digest printed by step 3. |
| `CDN_SIGNING_KEY_NAME` | The key name printed by step 8. |
| `PUBLIC_UPTIME_ENABLED` | `false` until step 9. |
| `CONTROL_PLANE_RUNTIME_READY` | Unset or `false` until a real runtime exists. |
| `CONTROL_PLANE_IMAGE` | Image pinned with `@sha256:` when runtime is ready. |

Limit GitHub deployment branches to those federation accepts. Keep the
launch switch at repository scope, without `--env`. Production also needs
the repository default branch to be `main`.

Upgrade `Firna-AI` to GitHub Team before production. Then a repository
admin installs the prepared ruleset:

```sh
bash deploy/scripts/github-ruleset.sh Firna-AI/ciproxy
```

It requires pull requests and the `Repository gate` check on `main`. It
blocks force pushes and deletion. It adds no required review count.

## 7. Run the first deploy

Push to the primary branch, or rerun all jobs of its last push run. After
that branch merges, use protected `main`. A normal run has this order:

1. Plan/apply seed and bootstrap when the caller can change trust.
2. Plan/apply shared core and this environment's central data root when
   the caller is its phase owner.
3. Build and deploy this environment's static files and Helm releases.
4. Wait for all configured NEGs. Save the actual workload inputs.
5. Plan/apply this environment's ready snapshot with separate identities.
6. Plan/apply this environment's central edge from its current immutable
   workload artifact when the caller is its phase owner.
7. Invalidate this environment's CDN and wait for completion.
8. Production then promotes preview data and its separate edge. It reads
   the exact successful preview run and attempt, never latest record state.
   A missing preview NEG cannot block production's completed invalidation.

An empty central promotion is a successful no-op. Its apply job uses only
the environment plan identity for status reads, then selects no roots.
It does not assume apply credentials, download a plan, or run Terraform.
Required apply jobs still succeed so successors can run normally.

The first run can stop until its image, token, or namespace exists.
Production data/edge state starts empty and prelaunch cannot write it.
Do not populate production secrets or the database until its data apply
passes. A preexisting name causes a create failure, not adoption. Follow
[state recovery](state.md#unexpected-existing-production-name) without
importing unknown production objects. Fix
that input and rerun all jobs. No apply stage creates a replacement plan.
Production follows only a successful preview push of the exact `main`
commit. See [promotion](promotion.md) for bindings and recovery.

## 8. Register the CDN signing key

After the first edge exists, register a random 128-bit key:

```sh
KEY_DIR="$(mktemp -d)"
head -c 16 /dev/urandom | base64 | tr +/ -_ > "$KEY_DIR/key"
bash deploy/scripts/register-cdn-key.sh "$DEPLOY_ENV" "$KEY_DIR/key"
rm -rf "$KEY_DIR"
```

The script adds secret version N, registers key
`ciproxy-<env>-images-v<N>` on that environment's backend bucket, and grants
the CDN cache-fill account read access to its private image bucket. Set
`CDN_SIGNING_KEY_NAME` only after the script prints it. A failed attempt
prints no selector. Retry with a new key. Terraform stores no key value.

## 9. Enable uptime alerts

Check DNS, HTTPS, and all hosts before setting
`PUBLIC_UPTIME_ENABLED=true` in this GitHub environment:

```sh
gcloud compute ssl-certificates describe "ciproxy-$DEPLOY_ENV" --global \
  --project=ciproxy --format='yaml(managed)'
curl -fsSI https://br-main.preview.ciproxy.dev/ https://br-main.preview.ciproxy.dev/docs/
curl -fsSI https://br-main.preview.app.ciproxy.dev/
curl -fsS https://br-main.preview.get.ciproxy.dev/ciproxy-agent | head -n 5
```

If the first real private-origin service-agent grant is refused by the
organization rule, stop that path and return item 76 with alternatives A,
B, and C. Do not widen organization policy automatically. See
[private-origin permissions](identities.md#private-static-origins).

Certificate status and each domain status must be `ACTIVE`. Redeploy after
changing the flag. Checks run each minute from Europe, USA, and Asia
Pacific. Alerts need two failing locations for five minutes.

After launch, a preview flag or address update reaches central edge on the
separate preview-edge stage after production's own completed deployment. It
never uses
production's flag or address for preview. Follow [operations](operations.md)
for job images, database grants, runtime enablement, and rotation.
