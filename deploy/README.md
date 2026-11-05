# Deployment

This directory deploys the hosted parts of ciproxy to Google Cloud: the
Terraform roots, the Helm charts for GKE Autopilot, and the scripts of the
operator and the deploy workflows. After one-time creation of the state
bucket and CI trust, CI plans, checks, and applies every Terraform root.
Run every command from the repository root.

The control-plane runtime is disabled. The control-plane binary is a CLI
skeleton with no HTTP service, so the deploy starts no control plane.
[Turn on the runtime](operations.md#turn-on-the-runtime) gives the
interface that a future image must meet.

- [Setup](setup.md): the identities, the permissions, and the commands of
  the one-time setup.
- [Operations](operations.md): the job images, the database, the
  control-plane runtime, the rotations, the zones, and later changes.
- [Terraform promotion](promotion.md): destructive approvals, saved plans,
  retries, stale locks, and operator recovery.

## Environments

| Environment | GCP project | Hosts | Deploys on |
| --- | --- | --- | --- |
| `main-preview` | `internal-498318` | `br-main.preview.ciproxy.dev`, `br-main.preview.app.ciproxy.dev`, `br-main.preview.get.ciproxy.dev` | Each push to `calummoore/ci-proxy-load-balancer`, then `main` after its merge |
| `production` | `ciproxy` | `ciproxy.dev`, `app.ciproxy.dev`, `get.ciproxy.dev` | A successful main preview push run on `main`, for that exact commit, when the repository variable `PRODUCTION_DEPLOY_ENABLED` is `true` |

Each environment has its own state, trust, accounts, cluster, database, and
load balancer. Every regional resource and every bucket is in
`europe-west1`. One global HTTPS load balancer serves the three hosts on one
IP address, with one Google-managed certificate. It redirects HTTP to HTTPS.

| Host and path | Backend |
| --- | --- |
| Site host | The site bucket through Cloud CDN. The docs are at `/docs`. |
| Dashboard host | The dashboard bucket through Cloud CDN. |
| Dashboard host: `/api`, `/agent`, `/hooks` | The control-plane backend. It has no endpoints while the runtime is disabled, so these paths fail. |
| Dashboard host: `/images` | The private image bucket through Cloud CDN, for signed URLs. Objects go under `images/`. |
| Download host | The `futex-ai/cli-release` server on GKE. |

The release server serves the install script and the `ciproxy-agent`
releases of `Firna-AI/ciproxy`, with the tag prefix `ciproxy-agent-v`. It
runs upstream commit `30513bd1a6e15d8e9c1b18f2ed92d1d4dfa11786`.
[PROVENANCE.json](helm/cli-release-server/PROVENANCE.json) records the
source and the file hashes of its vendored chart.

The release server and the future control plane run only in the zones
`europe-west1-b`, `europe-west1-c`, and `europe-west1-d`. GKE creates their
network endpoint groups (NEGs) in all three zones before the edge attaches
them. This NEG zone setting is a GKE Preview feature. It needs GKE
1.36.2-gke.3104000 or later, so the cluster has the minimum version 1.36.4.
[Zones and NEGs](operations.md#zones-and-negs) gives the operating rules.

The operator creates the state bucket and enables trust APIs in seed.
Bootstrap prepares all four remote states before CI starts. It leaves
existing states unchanged. The read-only plan
identity saves binary plans and checks their destructive actions. Only
`deploy-apply.yml` can use the separate owner-equivalent apply identity.
The workload identity deploys Helm and static files with its existing
roles. Runtime accounts pull images and serve the future control plane.
[Identities and permissions](setup.md#identities-and-permissions) gives
their boundaries.

## Layout

| Path | Contents | Applied by |
| --- | --- | --- |
| `deploy/terraform/seed/` | State bucket and trust API enablement, migrated to its own remote state | Operator once, then CI |
| `deploy/terraform/bootstrap/` | Federation providers, CI accounts, roles, and state grants | Operator once, then CI |
| `deploy/environments/<env>/` | VPC, NAT, GKE, private PostgreSQL 17, KMS, registry, secrets, buckets, DNS zones, and alert channels | CI |
| `deploy/environments/<env>/edge/` | Load balancer, CDN, certificate, records, uptime checks, and alerts | CI after Helm and the NEG wait |
| `deploy/terraform/modules/` | The `foundation` and `edge` modules of these roots | None |
| `deploy/helm/cli-release-server/` | The unchanged upstream chart | Deploy workflow |
| `deploy/helm/release-values.yaml` | The ciproxy values of the release server | Deploy workflow |
| `deploy/helm/ciproxy/` | The NEG Service of the release server, and the disabled control plane | Deploy workflow |
| `deploy/scripts/bootstrap.sh` | Creates seed locally, migrates it, prepares all four remote states, then creates CI trust | Operator once |
| `deploy/scripts/terraform.py` | Saves, checks, and applies exact binary plans | Plan identity or reusable apply workflow |
| `deploy/scripts/terraform_roots.py` | Shared root directories, state buckets, prefixes, and trust classification | Bootstrap, promotion, and workload commands |
| `deploy/scripts/terraform_policy.py` | Checks or generates the exact destructive approval | Plan and apply stages, or operator |
| `deploy/scripts/promotion_event.py` | Checks caller, repository, branch, event, and source commit | Every credentialed job |
| `deploy/scripts/github-ruleset.sh` | Creates or updates the `main` pull request and gate ruleset | Repository admin after the GitHub Team upgrade |
| `deploy/scripts/mirror-release-image.sh` | Pushes the release server image and prints its digest | Operator |
| `deploy/scripts/register-cdn-key.sh` | Stores a CDN signing key as a numbered secret version, and registers it under the matching name | Operator |
| `deploy/scripts/deploy.py` | The deploy command, with `deploy_support.py` and `publish_static.py` | Deploy workflow |
| `deploy/scripts/install-helm.sh` | Installs Helm 4.3.1 after a checksum check | Deploy workflow or operator |
| `deploy/tests/` | CLI tests, hand-worked policy cases, and recorded Terraform JSON | Offline check |
| `.github/workflows/deploy-<env>.yml` | Plan, policy check, infrastructure promotion, workloads, and edge plan | GitHub Actions |
| `.github/workflows/deploy-apply.yml` | The only workflow that can assume the apply identity | Called by an environment workflow |

The remote state is in the bucket `ciproxy-<env>-tfstate-<project>`, under
the prefixes `<env>/seed`, `<env>/bootstrap`, `<env>/foundation`, and
`<env>/edge`. The first local seed state remains as an operator backup at
`.context/deploy/<env>/seed.tfstate`. It is no longer the active state.

## Offline checks

These checks need no cloud credentials. `terraform init` downloads the
pinned `hashicorp/google` 7.12.0 provider into `.terraform/`, which Git
ignores. Use Terraform 1.17.1 and Helm 4.3.1.

```sh
terraform fmt -check -recursive deploy
terraform -chdir=deploy/terraform/seed init -backend=false -input=false
terraform -chdir=deploy/terraform/seed validate
terraform -chdir=deploy/terraform/bootstrap init -backend=false -input=false
terraform -chdir=deploy/terraform/bootstrap validate
terraform -chdir=deploy/environments/main-preview init -backend=false -input=false
terraform -chdir=deploy/environments/main-preview validate
terraform -chdir=deploy/environments/main-preview/edge init -backend=false -input=false
terraform -chdir=deploy/environments/main-preview/edge validate
terraform -chdir=deploy/environments/production init -backend=false -input=false
terraform -chdir=deploy/environments/production validate
terraform -chdir=deploy/environments/production/edge init -backend=false -input=false
terraform -chdir=deploy/environments/production/edge validate
terraform -chdir=deploy/terraform/bootstrap test
terraform -chdir=deploy/environments/main-preview test
helm lint deploy/helm/ciproxy
helm lint deploy/helm/cli-release-server --values deploy/helm/release-values.yaml
helm template cli-release-server deploy/helm/cli-release-server \
  --values deploy/helm/release-values.yaml
helm template ciproxy deploy/helm/ciproxy
helm template ciproxy deploy/helm/ciproxy --set controlPlane.enabled=true \
  --set controlPlane.image=example.invalid/control-plane:render-check \
  --set 'allowedZones={europe-west1-b,europe-west1-c,europe-west1-d}' \
  --set controlPlane.cdnSigningKeyName=ciproxy-main-preview-images-v1 \
  --set controlPlane.secrets.cdn-signing-key=projects/internal-498318/secrets/ciproxy-main-preview-cdn-signing-key
actionlint .github/workflows/deploy-main-preview.yml .github/workflows/deploy-production.yml .github/workflows/deploy-apply.yml
for script in deploy/scripts/*.sh; do bash -n "$script"; done
python3 -m py_compile deploy/scripts/*.py
python3 -m unittest discover -s deploy/tests -p 'test_*.py'
python3 deploy/scripts/deploy.py --help
npm --prefix ts run site:build
python3 deploy/scripts/publish_static.py --directory=ts/site/dist \
  --bucket=dry-run --dry-run
```

The default `ciproxy` render has no control plane. The runtime render uses
the first main preview CDN key, so its provider object lists secret version 1.
The last command lists each object and its headers, and uploads nothing.

## Setup checklist

Do every step for the main preview first, then for production. Each step
links to its commands and permissions.

1. [Prepare the project](setup.md#1-prepare-the-project): check billing and
   the organization policies, get the operator roles, install the tools, and
   log in with local credentials.
2. [Bootstrap the environment](setup.md#2-bootstrap-the-environment): create
   the state bucket, trust APIs, and CI trust. Migrate seed state and back
   it up. Prepare all four remote backends with operator credentials.
   Set the identity variables and `ALERT_EMAIL`, then start CI. The first
   run creates foundation and can stop at missing workload inputs.
3. [Push the release server image](setup.md#3-push-the-release-server-image):
   record the printed `RELEASE_SERVER_IMAGE_DIGEST`.
4. [Store the release token](setup.md#4-store-the-release-token): add a
   GitHub token for `Firna-AI/ciproxy` only to Secret Manager.
5. [Delegate DNS](setup.md#5-delegate-dns): add the three preview NS records
   to the active parent zone. During production setup, copy them before the
   registrar switch.
6. [Set up the GitHub environment](setup.md#6-set-up-the-github-environment):
   create it and set its variables.
7. [Run the first deploy](setup.md#7-run-the-first-deploy): start a new run
   with all jobs. Every root gets a new plan. Production follows the same
   `main` commit only after its main preview succeeds.
8. [Register the CDN signing key](setup.md#8-register-the-cdn-signing-key):
   then set `CDN_SIGNING_KEY_NAME` to the name that it prints.
9. [Turn on uptime alerts](setup.md#9-turn-on-uptime-alerts): check DNS,
   the certificate, and the hosts. Then set `PUBLIC_UPTIME_ENABLED` to
   `true`, and deploy again.
10. [Publish job images](operations.md#publish-job-images): when an image
    exists, upload it under `images/` and check the signed access.
11. [Prepare the database](operations.md#prepare-the-database): grant the
    runtime user its rights. Run the migrations when they exist.
12. [Turn on the runtime](operations.md#turn-on-the-runtime): only with a
    real control-plane image.

After the setup, [Operations](operations.md) gives the rotations of the
release token, the CDN key, and the runtime secrets. It also gives the
steps of a foundation change and of a zone change. Before production,
upgrade `Firna-AI` to GitHub Team and apply the prepared `main` ruleset.

## Related docs

- [Architecture](../docs/architecture.md): the components and the download
  site.
- [Key custody](../docs/protocol/security.md#key-custody): how ciproxy keeps
  the Boat keys.
- [ciproxy-control-plane](../crates/ciproxy-control-plane/README.md): the
  CLI skeleton that a runtime will replace.
