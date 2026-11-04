# Deployment setup

This page gives the one-time setup of one environment of the
[deployment](README.md). Do each step for staging first, then for
production. Run every command from the repository root, with the shell
variables of the environment:

```sh
DEPLOY_ENV=staging PROJECT_ID=internal-498318
# For production: DEPLOY_ENV=production PROJECT_ID=ciproxy
```

## Identities and permissions

| Identity | Used for | Permissions |
| --- | --- | --- |
| Operator: your own Google account | Steps 1 to 5, 8, and 9 here, and the image, database, runtime, rotation, and foundation work of [Operations](operations.md) | The Owner role, or these project roles: `roles/serviceusage.serviceUsageAdmin`, `roles/resourcemanager.projectIamAdmin`, `roles/iam.serviceAccountAdmin`, `roles/iam.serviceAccountUser`, `roles/iam.workloadIdentityPoolAdmin`, `roles/storage.admin`, `roles/compute.networkAdmin`, `roles/compute.securityAdmin`, `roles/compute.loadBalancerAdmin`, `roles/container.admin`, `roles/servicenetworking.networksAdmin`, `roles/cloudsql.admin`, `roles/secretmanager.admin`, `roles/cloudkms.admin`, `roles/artifactregistry.admin`, `roles/dns.admin`, and `roles/monitoring.editor` |
| Registrar or parent zone owner | Step 5 | Change the name servers of `ciproxy.dev`, or add an NS record to its zone |
| GitHub repository admin | Step 6 | Create environments and their variables in `Firna-AI/ciproxy` |
| GitHub writer | Step 7 and each later deploy | Push to `calummoore/ci-proxy-load-balancer`, or push a `deploy-*` tag |
| Image publisher | [Publish job images](operations.md#publish-job-images) | `roles/storage.objectCreator` on the image bucket, or the operator roles |
| Deploy identity: `ciproxy-<env>-deploy` | Steps 7 and 9, and each later deploy run, through Workload Identity Federation with no key | `roles/compute.loadBalancerAdmin`, `roles/compute.networkViewer`, `roles/container.developer`, `roles/dns.admin`, `roles/monitoring.editor`, and `roles/serviceusage.serviceUsageConsumer`. It reads each state and writes only the edge state. It writes the site and dashboard buckets, and reads the release token. |
| GKE nodes: `ciproxy-<env>-gke` | Image pulls | Read the runtime Artifact Registry repository |
| Runtime: `ciproxy-<env>-control` | The future control plane, through the Kubernetes service account `ciproxy/control-plane` | Cloud SQL login, the runtime secrets, and the Boat KMS key |

The deploy workflows never apply the bootstrap or foundation roots, and the
deploy identity cannot write their state. Its DNS and monitoring roles also
reach the zone and the alert channels of the foundation, but the scripts
change only edge resources. It has no admin role for Cloud SQL, KMS, the
network, or service accounts.

The federation provider accepts only a `push` run of `deploy-<env>.yml`
with the GitHub environment `<env>`, from the numeric repository and owner
IDs of `Firna-AI/ciproxy`. Staging also needs the ref
`refs/heads/calummoore/ci-proxy-load-balancer`, and production a ref that
starts with `refs/tags/deploy-`.

Secret values never go into Terraform, GitHub variables, Git, or a command
line. Give each one at a hidden prompt or in a private file outside Git.

## 1. Prepare the project

Check billing with `gcloud billing projects describe "$PROJECT_ID"`. The
site and dashboard buckets grant read access to `allUsers`, so an
organization policy for domain-restricted sharing or public access
prevention stops the foundation apply. Get the operator roles of the table.

Install the Google Cloud CLI with `kubectl` and `gke-gcloud-auth-plugin`,
Terraform 1.17.1, Helm 4.3.1, Docker, the GitHub CLI, Python 3, and `psql`.
The workflows use gcloud 592.0.0. `bash deploy/scripts/install-helm.sh`
puts Helm in `.context/tools/ciproxy-helm/`. Terraform uses the application
default credentials. Set the quota project again when you change the
environment.

```sh
gcloud auth login
gcloud auth application-default login
gcloud auth application-default set-quota-project "$PROJECT_ID"
```

## 2. Bootstrap the environment

```sh
bash deploy/scripts/bootstrap.sh "$DEPLOY_ENV" ALERT_EMAIL
```

`ALERT_EMAIL` receives the uptime alerts. The script enables the Storage,
Service Usage, and Resource Manager APIs. Then it applies the seed root with
local state, the bootstrap root, and the foundation root. Each apply shows
its plan and asks you to confirm. The script keeps the alert address in
`.context/deploy/<env>/operator.tfvars.json`. At the end, it prints the
GitHub variables, the deployment values, and the DNS name servers. Copy
`.context/deploy/<env>/seed.tfstate` to a secure backup outside Git.

The deploy workflows never apply the bootstrap or foundation roots. To read
their outputs or apply a change later, select the remote state of the
environment first. The bootstrap root serves both environments. On a new
machine, copy `operator.tfvars.example` to `operator.tfvars` in the same
directory, and pass `-var-file=operator.tfvars` instead. Git ignores it.

```sh
terraform -chdir=deploy/environments/$DEPLOY_ENV init -reconfigure \
  -backend-config="bucket=ciproxy-$DEPLOY_ENV-tfstate-$PROJECT_ID" \
  -backend-config="prefix=$DEPLOY_ENV/foundation"
terraform -chdir=deploy/environments/$DEPLOY_ENV apply \
  -var-file="$PWD/.context/deploy/$DEPLOY_ENV/operator.tfvars.json"
terraform -chdir=deploy/terraform/bootstrap init -reconfigure \
  -backend-config="bucket=ciproxy-$DEPLOY_ENV-tfstate-$PROJECT_ID" \
  -backend-config="prefix=$DEPLOY_ENV/bootstrap"
terraform -chdir=deploy/terraform/bootstrap apply \
  -var="environment=$DEPLOY_ENV" -var="project_id=$PROJECT_ID"
```

## 3. Push the release server image

The foundation creates the Artifact Registry repository
`ciproxy-<env>-runtime`. Docker must run locally.

```sh
gh auth login
bash deploy/scripts/mirror-release-image.sh "$DEPLOY_ENV"
```

By default, the script builds the pinned upstream commit from source. Your
GitHub login needs read access to `futex-ai/cli-release`. The script runs
`--help` in the image, pushes it, and prints
`RELEASE_SERVER_IMAGE_DIGEST=sha256:...`. When upstream has published the
image of that commit, the optional `published` mode mirrors it from
`ghcr.io` instead. That mode needs `read:packages`.

## 4. Store the release token

Create a fine-grained GitHub token for `Firna-AI/ciproxy` only, with the
Contents permission read-only. An organization owner may need to approve
it. The hidden prompt keeps it out of the screen, the history, and the
process list. Each deploy copies the latest version into the Kubernetes
Secret `ciproxy/release-github-token` through standard input.

```sh
read -rs RELEASE_TOKEN
printf '%s' "$RELEASE_TOKEN" | gcloud secrets versions add \
  "ciproxy-$DEPLOY_ENV-release-github-token" --project="$PROJECT_ID" --data-file=-
unset RELEASE_TOKEN
```

## 5. Delegate DNS

The foundation creates the zone `ciproxy-<env>`: `ciproxy.dev` for
production and `staging.ciproxy.dev` for staging. The first deploy adds the
A records. Terraform never changes the registrar or a parent zone. Get the
name servers with
`terraform -chdir=deploy/environments/$DEPLOY_ENV output dns_name_servers`.

- **Production:** first copy each record that `ciproxy.dev` still needs,
  such as MX and TXT, into the zone. The zone has no DNSSEC, so remove any
  DS record. Then set the four name servers at the registrar.
- **Staging:** add an NS record for `staging.ciproxy.dev` in the zone that
  serves `ciproxy.dev` at that time. Before a later production switch, copy
  it into the production zone. With `roles/dns.admin` in `ciproxy`, run:

```sh
gcloud dns record-sets create staging.ciproxy.dev. --project=ciproxy \
  --zone=ciproxy-production --type=NS --ttl=3600 --rrdatas=NS1,NS2,NS3,NS4
```

## 6. Set up the GitHub environment

The workflows read the variables of the GitHub environment `<env>`, and no
GitHub secrets. Create the environment, read the bootstrap output, and set
each variable of the table with `gh variable set`:

```sh
gh api --method PUT "repos/Firna-AI/ciproxy/environments/$DEPLOY_ENV"
terraform -chdir=deploy/terraform/bootstrap output -json github_environment_variables
gh variable set GCP_WIF_PROVIDER --repo=Firna-AI/ciproxy --env="$DEPLOY_ENV" --body=VALUE
```

| Variable | Value | When |
| --- | --- | --- |
| `GCP_WIF_PROVIDER` | The bootstrap output | Before the first deploy |
| `GCP_DEPLOY_SERVICE_ACCOUNT` | The bootstrap output | Before the first deploy |
| `RELEASE_SERVER_IMAGE_DIGEST` | The `sha256:` digest of step 3 | Before the first deploy |
| `CDN_SIGNING_KEY_NAME` | The key name of step 8 | After step 8 |
| `PUBLIC_UPTIME_ENABLED` | `true` | Step 9 |
| `CONTROL_PLANE_RUNTIME_READY` | Unset or `false`, until a real runtime exists | [Runtime](operations.md#turn-on-the-runtime) |
| `CONTROL_PLANE_IMAGE` | An image name with `@sha256:` and its digest | [Runtime](operations.md#turn-on-the-runtime) |

The deploy stops before its first cloud call when the digest is not
`sha256:` with 64 hex digits, or when `CONTROL_PLANE_RUNTIME_READY` is not
`true`, `false`, or unset. Optionally, limit the deployment branches and
tags of the environment to the ref that the trust accepts.

## 7. Run the first deploy

Do steps 2 to 6 first. DNS can still be pending. For staging, push to
`calummoore/ci-proxy-load-balancer`, or run the last "Deploy staging" run
again. For production, push a new tag:

```sh
git tag deploy-20261104-1 COMMIT
git push origin deploy-20261104-1
```

The workflow builds the site and the dashboard, logs in through federation,
and runs `python3 deploy/scripts/deploy.py <env>`. That command copies the
release token into Kubernetes. It upgrades the Helm releases
`cli-release-server` and `ciproxy` in the namespace `ciproxy`, and uploads
the static files without a delete. It waits until GKE reports the NEGs of
each Service in all three zones. Then it applies the edge root, which
attaches the NEGs of all three zones, and invalidates the CDN cache. When a
zone is still missing after ten minutes, the deploy stops before the edge
apply. Until steps 2 to 6 are done, each staging run fails.

## 8. Register the CDN signing key

The first deploy creates the backend bucket `ciproxy-<env>-images`. Make a
random 128-bit key in a private directory, register it, and delete the file.
Use a new key name each time, such as `ciproxy-staging-images-1`.

```sh
KEY_DIR="$(mktemp -d)"
head -c 16 /dev/urandom | base64 | tr +/ -_ > "$KEY_DIR/key"
bash deploy/scripts/register-cdn-key.sh "$DEPLOY_ENV" KEY_NAME "$KEY_DIR/key"
rm -rf "$KEY_DIR"
```

The script registers the key on the backend bucket. It lets the Cloud CDN
cache-fill account read the image bucket, and adds the key to the secret
`ciproxy-<env>-cdn-signing-key`. Set `CDN_SIGNING_KEY_NAME` to the key name.
Only the future runtime signs URLs.

## 9. Turn on uptime alerts

After the delegation and the first deploy, check DNS, the certificate, and
the hosts. The example uses the staging hosts. The certificate is ready when
its status and each domain status are `ACTIVE`. Browsers open `.dev` hosts
over HTTPS only, so no page loads before then.

```sh
dig +short NS staging.ciproxy.dev
gcloud compute ssl-certificates describe "ciproxy-$DEPLOY_ENV" --global \
  --project="$PROJECT_ID" --format="yaml(managed)"
curl -fsSI https://staging.ciproxy.dev/ https://staging.ciproxy.dev/docs/
curl -fsSI https://app.staging.ciproxy.dev/
curl -fsS https://get.staging.ciproxy.dev/ciproxy-agent | head -n 5
```

Then set `PUBLIC_UPTIME_ENABLED` to `true` and deploy again. The edge adds
uptime checks for `/` and `/docs/` on the site host, `/` on the `app.` host,
and `/ciproxy-agent` on the `get.` host. Each check runs each minute from
Europe, the USA, and Asia Pacific. Its alert policy sends an email to the
alert address when two or more checker locations fail for five minutes.

The job images, the database, and the control-plane runtime come later.
See [Operations](operations.md).
