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

Install Terraform 1.17.1, Google Cloud CLI, kubectl,
`gke-gcloud-auth-plugin`, Docker, Git, GitHub CLI, Python 3, jq, psql,
Bash, curl, and tar before using the commands below. CI uses Google Cloud
CLI 592.0.0. Install Helm 4.3.1 with the pinned checksum installer:

```sh
bash deploy/scripts/install-helm.sh
export PATH="$PWD/.context/tools/ciproxy-helm:$PATH"
gcloud auth login
gcloud auth application-default login
gcloud auth application-default set-quota-project "$PROJECT_ID"
gh auth login
gcloud billing projects describe "$PROJECT_ID"
```

Check billing and organization policies with the authenticated operator.
Static origins enforce PAP and need no public grant. Their two Google
agents need the managed sharing rule of project `ciproxy`. An Organization
Policy Administrator applies it once, the managed rule first:

```sh
gcloud services enable orgpolicy.googleapis.com --project="$PROJECT_ID"
gcloud org-policies set-policy deploy/org-policies/allowed-policy-members.yaml
gcloud org-policies set-policy deploy/org-policies/legacy-domains-off.yaml
```

The edge later grants the load balancer agent viewer access after cached
backend creation. See [operator roles](identities.md#operator-permissions)
and [static delivery](static-delivery.md).

## 2. Bootstrap

Keep the repository production switch false. Create initial seed and trust:

```sh
gh variable set PRODUCTION_DEPLOY_ENABLED --repo=Firna-AI/ciproxy --body=false
bash deploy/scripts/bootstrap.sh main-preview
```

Seed creates four versioned state buckets and enables trust APIs. The
script uses a temporary local backend until the state bucket exists. It
then asks Terraform to migrate seed state to `trust/seed`. Review both
locations. It never forces an old local copy over remote state.

Bootstrap prepares all nine remote backends and both environments' CI and
runtime identities. It creates no SQL, application secrets/buckets, zones,
or workloads. Back up `.context/deploy/ciproxy/seed.tfstate` outside Git.
On another operator machine, use `bootstrap.sh main-preview --existing-state`
to read existing state without another local seed apply.

Before any preview push, complete [operator UID setup](runtime-identity.md#first-shared-core-and-preview-identity).
It creates the intended Autopilot core, preview namespace, and runtime
ServiceAccount. It records the real UID through protected bootstrap state
and installs the preview binding. Prelaunch CI cannot create/delete clusters
or attach the node account. Routine exact-cluster updates stay automatic.

Both Namespace manifests enforce/audit/warn at Baseline. Only the operator
owns these labels and namespace RBAC. The network policy permits traffic
inside each namespace and blocks other workload namespaces. Reapply existing
namespaces as [the admission procedure](identities.md#baseline-admission)
describes. Future trust updates on protected `main` preserve the UID map.
Preview artifacts cannot supply production UIDs.

## 3. Push the release image

The UID procedure created registry `ciproxy-shared-runtime`. Start Docker.
The script builds pinned upstream source, checks its help command, pushes
the image, and prints `RELEASE_SERVER_IMAGE_DIGEST`:

```sh
bash deploy/scripts/mirror-release-image.sh "$DEPLOY_ENV"
```

The operator GitHub account needs read access to `futex-ai/cli-release`.
Optional `published` mode uses its published image and needs `read:packages`.
Both environments can use the same immutable digest.

## 4. Set GitHub variables

Create the selected GitHub environment. Read its twelve identity values
from the already-applied bootstrap output:

```sh
gh api --method PUT "repos/Firna-AI/ciproxy/environments/$DEPLOY_ENV"
terraform -chdir=deploy/terraform/bootstrap output -json github_environment_variables \
  | jq --arg env "$DEPLOY_ENV" '.[$env]'
gh variable set GCP_WIF_PROVIDER --repo=Firna-AI/ciproxy \
  --env="$DEPLOY_ENV" --body=VALUE
```

Set every printed identity variable in that environment. They cover
workload, environment plan/apply, central plan/apply, and central trust
apply. Never copy preview's central apply identities into production.

| Variable | Initial value |
| --- | --- |
| `ALERT_EMAIL` | This environment's alert address. |
| `RELEASE_SERVER_IMAGE_DIGEST` | The mirrored `sha256:` digest from [image setup](#3-push-the-release-image). |
| `PUBLIC_UPTIME_ENABLED` | `false` until [HTTPS verification](#10-enable-uptime-alerts). |
| `CONTROL_PLANE_RUNTIME_READY` | `false`. Keep it off until a real runtime exists. |
| `CONTROL_PLANE_IMAGE` | Unset while runtime is off. Later use an image pinned with `@sha256:`. |
| `CDN_SIGNING_KEY_NAME` | **Unset** until [registration](#9-register-the-cdn-signing-key) prints the real key name. |

Limit deployment branches to those federation accepts. Keep the launch
switch at repository scope, without `--env`. Production needs default
branch `main`. Upgrade `Firna-AI` to GitHub Team before production. A
repository admin then installs the prepared ruleset:

```sh
bash deploy/scripts/github-ruleset.sh Firna-AI/ciproxy
```

It requires pull requests and `Repository gate` on `main`. It blocks force
pushes and deletion. It adds no required review count.

## 5. Create preview data through CI

Only now start a preview push run of the current deployment code. Follow
[start the first preview run](first-deploy.md#start-the-first-preview-run)
for exact run selection and all-jobs rerun commands. There is no manual
preview-data apply or secret/zone creation.

Infrastructure must finish `apply-data` and its aggregate successfully.
The workload job then validates the digest, reads data outputs, and opens
the namespace. With no token version, its latest-version read fails before
Helm. This is the expected first stop. The edge does not exist yet.

Use [confirm the data boundary](first-deploy.md#confirm-the-data-boundary)
to inspect the run, release-token parent, and all three preview zones.
Confirm no token version exists. An infrastructure, identity, permission,
image, or namespace error is not the expected stop. Resolve it first.

## 6. Store the release token

Continue only after CI created and you confirmed the parent secret.
Create a fine-grained GitHub token with read-only access to public
repositories only. `Firna-AI/ciproxy` and its release assets are public.
The release server still needs a token for its GitHub API configuration.
It needs no private repository access. Enter the token at a hidden prompt:

```sh
read -rs RELEASE_TOKEN
printf '%s' "$RELEASE_TOKEN" | gcloud secrets versions add \
  "ciproxy-$DEPLOY_ENV-release-github-token" --project=ciproxy --data-file=-
unset RELEASE_TOKEN
```

Deploy later copies it through stdin to
`ciproxy-<env>/release-github-token`. Terraform never reads its value.

## 7. Delegate preview DNS

The successful data apply created preview's three Google zones. Follow
[main preview DNS](dns.md#main-preview) to read their name servers and add
Cloudflare NS sets named `preview`, `preview.app`, and `preview.get`.
Keep Cloudflare's apex, Workspace mail, registrar NS, and DNSSEC unchanged.
Keep the preview NS sets after creating them.

Production parent/apex records belong to the [launch procedure](launch.md).
Do not add `app`, `get`, or an apex A record during preview setup.
There is no Cloudflare provider or CI credential.

## 8. Complete the first deploy

After token storage and preview delegation, [rerun all jobs](first-deploy.md#retry-all-jobs-and-create-the-edge)
of the selected push run. It creates fresh plans and artifacts for the new
attempt. The run can now deploy Helm/static content, wait for NEGs, save
its ready snapshot, apply the edge, and invalidate CDN. The edge creates
its A records, certificate, and backends. Certificate readiness comes later.

The first private-origin service-agent grant happens here. It needs the
managed sharing rule of [project preparation](#1-prepare-the-project). See
[private-origin permissions](identities.md#private-static-origins).
Other failures follow [promotion recovery](promotion.md). No apply stage
creates a replacement plan. Empty central scopes remain successful no-ops.

## 9. Register the CDN signing key

Continue only after the edge apply succeeded. Its image backend, data
signing-key secret, and private image bucket now exist. Register a random
128-bit key:

```sh
KEY_DIR="$(mktemp -d)"
head -c 16 /dev/urandom | base64 | tr +/ -_ > "$KEY_DIR/key"
bash deploy/scripts/register-cdn-key.sh "$DEPLOY_ENV" "$KEY_DIR/key"
rm -rf "$KEY_DIR"
```

The script adds secret version N, registers `ciproxy-<env>-images-v<N>`,
then grants the newly created cache-fill account read access to images.
Set `CDN_SIGNING_KEY_NAME` to its printed name only after success. A failed
attempt prints no selector. Retry registration with a new key. Terraform
stores no key value. Runtime remains off until [runtime setup](runtime.md).

## 10. Enable uptime alerts

Confirm delegated DNS and the created certificate before enabling alerts:

```sh
gcloud compute ssl-certificates describe "ciproxy-$DEPLOY_ENV" --global \
  --project=ciproxy --format='yaml(managed)'
```

For production, add its [post-edge apex A record](dns.md#apex-site-record)
first. Select this environment's hosts, then check HTTPS:

```sh
case "$DEPLOY_ENV" in
  main-preview)
    SITE_HOST=br-main.preview.ciproxy.dev
    DASHBOARD_HOST=br-main.preview.app.ciproxy.dev
    RELEASE_HOST=br-main.preview.get.ciproxy.dev ;;
  production)
    SITE_HOST=ciproxy.dev
    DASHBOARD_HOST=app.ciproxy.dev
    RELEASE_HOST=get.ciproxy.dev ;;
esac
curl -fsSI "https://$SITE_HOST/" "https://$SITE_HOST/docs/"
curl -fsSI "https://$DASHBOARD_HOST/"
curl -fsS "https://$RELEASE_HOST/ciproxy-agent" | head -n 5
```

Certificate and domain status must be `ACTIVE`. Set
`PUBLIC_UPTIME_ENABLED=true` in the selected environment.
Redeploy with fresh all-jobs plans. Checks run each minute from Europe,
USA, and Asia Pacific. Alerts need two failing locations for five minutes.

After launch, production's separate preview-edge stage promotes preview's
flags and addresses after its own completed deployment. See [operations](operations.md).
