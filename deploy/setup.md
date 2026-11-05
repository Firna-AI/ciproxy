# Deployment setup

This page gives the one-time setup of one environment of the
[deployment](README.md). Do each step for the main preview first, then for
production. Run every command from the repository root, with the shell
variables of the environment:

```sh
DEPLOY_ENV=main-preview PROJECT_ID=internal-498318
# For production: DEPLOY_ENV=production PROJECT_ID=ciproxy
```

## Identities and permissions

Read [Deployment identities](identities.md) for operator roles, CI roles,
federation claims, and the runtime accounts. The apply account can change
project IAM. Only the reusable apply workflow can use that account.

## 1. Prepare the project

Check billing with `gcloud billing projects describe "$PROJECT_ID"`. The
site and dashboard buckets grant read access to `allUsers`, so an
organization policy for domain-restricted sharing or public access
prevention stops the foundation apply. Get the operator roles in the
identity guide.

A zone on the last free Cloud DNS name-server set needs a TXT ownership check.
Complete the check before retrying setup. See the
[DNS limits](https://docs.cloud.google.com/dns/quotas#name_server_limits).

Install the Google Cloud CLI with `kubectl` and `gke-gcloud-auth-plugin`,
Terraform 1.17.1, Helm 4.3.1, Docker, the GitHub CLI, Python 3, and `psql`.
The workflows use gcloud 592.0.0. The Helm installer supports Linux and
macOS on amd64 and arm64. It selects the native archive and checks its
pinned SHA-256 before extraction. It uses `sha256sum` or `shasum -a 256`.
The pins come from the
[Helm 4.3.1 release](https://github.com/helm/helm/releases/tag/v4.3.1).
The installer puts Helm in `.context/tools/ciproxy-helm/`. It adds that
directory to `GITHUB_PATH` in CI. For an operator, add it to `PATH`:

```sh
bash deploy/scripts/install-helm.sh
export PATH="$PWD/.context/tools/ciproxy-helm:$PATH"
gcloud auth login
gcloud auth application-default login
gcloud auth application-default set-quota-project "$PROJECT_ID"
```

Terraform uses the application default credentials. Set the quota project
again when you change the environment.

## 2. Bootstrap the environment

```sh
bash deploy/scripts/bootstrap.sh "$DEPLOY_ENV"
```

The script enables the three APIs needed before Terraform can run. It
creates the state bucket from a temporary local-backend copy of the seed.
Then it migrates that state to `<env>/seed` in the same bucket. This order
avoids a dependency on a bucket that does not exist. The tracked seed root
always uses GCS. Finally, the script applies bootstrap and prints the six
GitHub identity variables. Both first applies ask for confirmation.
Terraform also asks before it migrates state. Inspect its source and
destination. The script never forces a copy over existing remote state.

The script can adopt the former `.context/deploy/<env>/seed.tfstate`.
After migration, a repeated call uses remote seed state. Back up the local
copy outside Git. Do not restore it over newer remote state.

Set the six identity variables and `ALERT_EMAIL` from step 6 now. Start a
main preview run. CI applies seed, bootstrap, and foundation before it
checks the release image digest. The first run can stop at that missing
digest. This is the setup point for steps 3 to 5. No local foundation apply
is needed. Production follows this same sequence when its launch switch
first permits a run. Its first run can stop after foundation creation.

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

The main preview foundation creates three zones in `internal-498318`:

| Managed zone | DNS name |
| --- | --- |
| `ciproxy-main-preview` | `preview.ciproxy.dev` |
| `ciproxy-main-preview-dashboard` | `preview.app.ciproxy.dev` |
| `ciproxy-main-preview-releases` | `preview.get.ciproxy.dev` |

Terraform leaves the registrar and delegations to the operator. The first
deploy adds `br-main` A records. Read each zone's four name servers:

```sh
terraform -chdir=deploy/environments/$DEPLOY_ENV output -json dns_name_servers
```

The output maps full DNS names with trailing dots to their name servers.

For the main preview, find the active parent zone in project `ciproxy`:

```sh
gcloud dns managed-zones list --project=ciproxy --filter='dnsName=ciproxy.dev.' \
  --format='table(name,nameServers)'
dig +short NS ciproxy.dev
PARENT_ZONE=ACTIVE_ZONE_ID
```

Set `PARENT_ZONE` to the `NAME` whose name servers match the `dig` output.
Use each preview zone's servers for its `SITE_NS`, `APP_NS`, or `GET_NS`.

```sh
gcloud dns record-sets create preview.ciproxy.dev. --project=ciproxy \
  --zone="$PARENT_ZONE" --type=NS --ttl=3600 \
  --rrdatas=SITE_NS1,SITE_NS2,SITE_NS3,SITE_NS4
gcloud dns record-sets create preview.app.ciproxy.dev. --project=ciproxy \
  --zone="$PARENT_ZONE" --type=NS --ttl=3600 \
  --rrdatas=APP_NS1,APP_NS2,APP_NS3,APP_NS4
gcloud dns record-sets create preview.get.ciproxy.dev. --project=ciproxy \
  --zone="$PARENT_ZONE" --type=NS --ttl=3600 \
  --rrdatas=GET_NS1,GET_NS2,GET_NS3,GET_NS4
```

After production foundation setup, copy the active zone's required records
to `ciproxy-production` in `ciproxy`. Include MX, TXT, and the three preview
NS records unchanged. Use `PARENT_ZONE=ciproxy-production` above. The new
zone has no DNSSEC. Remove any DS record before the registrar switch. Then
set the registrar to the four name servers in the `ciproxy.dev.` output.

These zones permit later wildcard records without another parent-zone edit.

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
| `GCP_PLAN_WIF_PROVIDER` | The bootstrap output | Before foundation creation |
| `GCP_PLAN_SERVICE_ACCOUNT` | The bootstrap output | Before foundation creation |
| `GCP_APPLY_WIF_PROVIDER` | The bootstrap output | Before foundation creation |
| `GCP_APPLY_SERVICE_ACCOUNT` | The bootstrap output | Before foundation creation |
| `ALERT_EMAIL` | The address for uptime alerts | Before foundation creation |
| `RELEASE_SERVER_IMAGE_DIGEST` | The `sha256:` digest of step 3 | Before the first deploy |
| `CDN_SIGNING_KEY_NAME` | The name that step 8 prints, such as `ciproxy-main-preview-images-v1` | After step 8 |
| `PUBLIC_UPTIME_ENABLED` | `true` | Step 9 |
| `CONTROL_PLANE_RUNTIME_READY` | Unset or `false`, until a real runtime exists | [Runtime](operations.md#turn-on-the-runtime) |
| `CONTROL_PLANE_IMAGE` | An image name with `@sha256:` and its digest | [Runtime](operations.md#turn-on-the-runtime) |

The workload stage stops before its cloud calls when the digest is not
`sha256:` with 64 hex digits, or runtime readiness is invalid. The preceding
infrastructure stage can still create foundation. Limit environment
deployment branches to the refs that federation accepts.

Set the launch switch as a **repository variable**, without `--env`:

```sh
gh variable set PRODUCTION_DEPLOY_ENABLED --repo=Firna-AI/ciproxy --body=false
```

The workflow reads this switch before it selects a GitHub environment.
At launch, set it to `true`. Production also needs the default branch to
be `main`, with both deploy workflows present there. GitHub runs
`workflow_run` workflows from the default branch.

The owner upgrades `Firna-AI` to GitHub Team. Then a repository admin runs:

```sh
bash deploy/scripts/github-ruleset.sh Firna-AI/ciproxy
```

The idempotent script creates or updates one ruleset on `main`. It requires
a pull request and the GitHub Actions `Repository gate` check from
`.github/workflows/ci.yml`. It forbids force pushes and deletion. It adds
no required review count and no up-to-date-branch requirement.

## 7. Run the first deploy

After foundation creation, finish steps 3 to 6. DNS can still be pending.
Push to `calummoore/ci-proxy-load-balancer`, or rerun **all jobs** of its
last run. After that branch merges into `main`, stop pushes to it and use
`main`. The workflow and federation accept both refs, so the merge needs
no trust change. This change adds no pull request preview environment.
Production follows only a successful main preview push run on `main`.
Its checks verify the source repository, workflow, event, branch, and
commit. Each checkout selects that run's commit, even if `main` moves.

Each environment plans and checks seed, bootstrap, and foundation. The
reusable workflow applies those exact binaries in that order. The workload
stage builds the static exports, copies the release token into Kubernetes,
upgrades both Helm releases, and uploads static files without deletion.
It waits for every configured NEG zone. Then the read-only identity saves
and checks the edge plan. The same reusable workflow applies it and
invalidates CDN. A failed NEG wait stops before the edge plan.
See [retries and recovery](promotion.md) before restarting a failed run.

## 8. Register the CDN signing key

The first deploy creates the backend bucket `ciproxy-<env>-images`. Make a
random 128-bit key in a private directory, register it, and delete the file:

```sh
KEY_DIR="$(mktemp -d)"
head -c 16 /dev/urandom | base64 | tr +/ -_ > "$KEY_DIR/key"
bash deploy/scripts/register-cdn-key.sh "$DEPLOY_ENV" "$KEY_DIR/key"
rm -rf "$KEY_DIR"
```

The script first adds the key file as a new version of the secret
`ciproxy-<env>-cdn-signing-key`. It reads only the number N of that
version, and makes the key name `ciproxy-<env>-images-v<N>`. Then it
registers the key under that name on the backend bucket, and lets the Cloud
CDN cache-fill account read the image bucket. Only when each step succeeds,
it prints the name, such as
`CDN_SIGNING_KEY_NAME=ciproxy-main-preview-images-v1` for the first main
preview key. Set the GitHub variable `CDN_SIGNING_KEY_NAME` to that value.

This one name selects both the key name and the secret version of the
future runtime. When the script stops early, it prints no name, so no deploy
uses that version. Then run it again with a new key. Only the future runtime
signs URLs.

## 9. Turn on uptime alerts

After the delegation and the first deploy, check DNS, the certificate, and
the hosts. The example uses the main preview hosts. The certificate is
ready when its status and each domain status are `ACTIVE`. Browsers open
`.dev` hosts over HTTPS only, so no page loads before then.

```sh
dig +short NS preview.ciproxy.dev
dig +short NS preview.app.ciproxy.dev
dig +short NS preview.get.ciproxy.dev
gcloud compute ssl-certificates describe "ciproxy-$DEPLOY_ENV" --global \
  --project="$PROJECT_ID" --format="yaml(managed)"
curl -fsSI https://br-main.preview.ciproxy.dev/ https://br-main.preview.ciproxy.dev/docs/
curl -fsSI https://br-main.preview.app.ciproxy.dev/
curl -fsS https://br-main.preview.get.ciproxy.dev/ciproxy-agent | head -n 5
```

Then set `PUBLIC_UPTIME_ENABLED` to `true` and deploy again. The edge adds
uptime checks for `/` and `/docs/` on the site host, `/` on the dashboard host,
and `/ciproxy-agent` on the download host. Each check runs each minute from
Europe, the USA, and Asia Pacific. Its alert policy sends an email to the
alert address when two or more checker locations fail for five minutes.

The job images, the database, and the control-plane runtime come later.
See [Operations](operations.md).
