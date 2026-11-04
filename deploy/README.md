# Deployment

This directory deploys the hosted parts of ciproxy to Google Cloud: the
Terraform roots, the Helm charts for GKE Autopilot, and the scripts of the
operator and the deploy workflows. Only the two deploy workflows run by
themselves. Each one-time step is a manual action of an operator. Run every
command from the repository root.

The control-plane runtime is disabled. The control-plane binary is a CLI
skeleton with no HTTP service, so the deploy starts no control plane.
[Turn on the runtime](operations.md#turn-on-the-runtime) gives the
interface that a future image must meet.

- [Setup](setup.md): the identities, the permissions, and the commands of
  the one-time setup.
- [Operations](operations.md): the job images, the database, the
  control-plane runtime, the rotations, the zones, and later changes.

## Environments

| Environment | GCP project | Hosts | Deploys on |
| --- | --- | --- | --- |
| `staging` | `internal-498318` | `staging.ciproxy.dev`, `app.staging.ciproxy.dev`, `get.staging.ciproxy.dev` | Each push to `calummoore/ci-proxy-load-balancer` |
| `production` | `ciproxy` | `ciproxy.dev`, `app.ciproxy.dev`, `get.ciproxy.dev` | Each push of a `deploy-*` tag |

Each environment has its own state, trust, accounts, cluster, database, and
load balancer. Every regional resource and every bucket is in
`europe-west1`. One global HTTPS load balancer serves the three hosts on one
IP address, with one Google-managed certificate. It redirects HTTP to HTTPS.

| Host and path | Backend |
| --- | --- |
| Site host | The site bucket through Cloud CDN. The docs are at `/docs`. |
| `app.` host | The dashboard bucket through Cloud CDN. |
| `app.` host: `/api`, `/agent`, `/hooks` | The control-plane backend. It has no endpoints while the runtime is disabled, so these paths fail. |
| `app.` host: `/images` | The private image bucket through Cloud CDN, for signed URLs. Objects go under `images/`. |
| `get.` host | The `futex-ai/cli-release` server on GKE. |

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

Three kinds of identity act on a project. The operator applies the seed,
bootstrap, and foundation roots with local credentials. The deploy identity
applies the edge in each deploy run, through Workload Identity Federation
with no key. The runtime accounts pull images and serve the future control
plane. [Identities and permissions](setup.md#identities-and-permissions)
gives their roles.

## Layout

| Path | Contents | Applied by |
| --- | --- | --- |
| `deploy/terraform/seed/` | The state bucket, with local state | Operator, once |
| `deploy/terraform/bootstrap/` | The Workload Identity Federation pool and provider, and the deploy service account | Operator |
| `deploy/environments/<env>/` | The foundation root: VPC, NAT, GKE Autopilot, Cloud SQL for PostgreSQL 17, the Boat KMS key, Artifact Registry, secrets, buckets, the DNS zone, and alert email channels | Operator |
| `deploy/environments/<env>/edge/` | The edge root: load balancer, Cloud CDN, certificate, A records, uptime checks, and alerts | Deploy workflow |
| `deploy/terraform/modules/` | The `foundation` and `edge` modules of these roots | None |
| `deploy/helm/cli-release-server/` | The unchanged upstream chart | Deploy workflow |
| `deploy/helm/release-values.yaml` | The ciproxy values of the release server | Deploy workflow |
| `deploy/helm/ciproxy/` | The NEG Service of the release server, and the disabled control plane | Deploy workflow |
| `deploy/scripts/bootstrap.sh` | Applies the seed, bootstrap, and foundation roots | Operator |
| `deploy/scripts/mirror-release-image.sh` | Pushes the release server image and prints its digest | Operator |
| `deploy/scripts/register-cdn-key.sh` | Stores a CDN signing key as a numbered secret version, and registers it under the matching name | Operator |
| `deploy/scripts/deploy.py` | The deploy command, with `deploy_support.py` and `publish_static.py` | Deploy workflow |
| `deploy/scripts/install-helm.sh` | Installs Helm 4.3.1 after a checksum check | Deploy workflow or operator |
| `deploy/tests/` | The regression tests of the NEG zone wait and the static publisher | Offline check |
| `.github/workflows/deploy-<env>.yml` | The deploy workflow of each environment | GitHub Actions |

The remote state is in the bucket `ciproxy-<env>-tfstate-<project>`, under
the prefixes `<env>/bootstrap`, `<env>/foundation`, and `<env>/edge`. The
seed state stays local, at `.context/deploy/<env>/seed.tfstate`.

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
terraform -chdir=deploy/environments/staging init -backend=false -input=false
terraform -chdir=deploy/environments/staging validate
terraform -chdir=deploy/environments/staging/edge init -backend=false -input=false
terraform -chdir=deploy/environments/staging/edge validate
terraform -chdir=deploy/environments/production init -backend=false -input=false
terraform -chdir=deploy/environments/production validate
terraform -chdir=deploy/environments/production/edge init -backend=false -input=false
terraform -chdir=deploy/environments/production/edge validate
helm lint deploy/helm/ciproxy
helm lint deploy/helm/cli-release-server --values deploy/helm/release-values.yaml
helm template cli-release-server deploy/helm/cli-release-server \
  --values deploy/helm/release-values.yaml
helm template ciproxy deploy/helm/ciproxy
helm template ciproxy deploy/helm/ciproxy --set controlPlane.enabled=true \
  --set controlPlane.image=example.invalid/control-plane:render-check \
  --set 'allowedZones={europe-west1-b,europe-west1-c,europe-west1-d}' \
  --set controlPlane.cdnSigningKeyName=ciproxy-staging-images-v1 \
  --set controlPlane.secrets.cdn-signing-key=projects/internal-498318/secrets/ciproxy-staging-cdn-signing-key
actionlint .github/workflows/deploy-staging.yml .github/workflows/deploy-production.yml
for script in deploy/scripts/*.sh; do bash -n "$script"; done
python3 -m py_compile deploy/scripts/*.py
python3 -m unittest deploy/tests/test_neg_readiness.py deploy/tests/test_publish_static.py
python3 deploy/scripts/deploy.py --help
npm --prefix ts run site:build
python3 deploy/scripts/publish_static.py --directory=ts/site/dist \
  --bucket=dry-run --dry-run
```

The default `ciproxy` render has no control plane. The runtime render uses
the first staging CDN key, so its provider object lists secret version 1.
The last command lists each object and its headers, and uploads nothing.

## Setup checklist

Do every step for staging first, then for production. Each step links to
its commands and permissions.

1. [Prepare the project](setup.md#1-prepare-the-project): check billing and
   the organization policies, get the operator roles, install the tools, and
   log in with local credentials.
2. [Bootstrap the environment](setup.md#2-bootstrap-the-environment): run
   `deploy/scripts/bootstrap.sh`. It creates the state bucket, the trust,
   and the foundation. Back up the seed state outside Git.
3. [Push the release server image](setup.md#3-push-the-release-server-image):
   record the printed `RELEASE_SERVER_IMAGE_DIGEST`.
4. [Store the release token](setup.md#4-store-the-release-token): add a
   GitHub token for `Firna-AI/ciproxy` only to Secret Manager.
5. [Delegate DNS](setup.md#5-delegate-dns): set the production name servers
   at the registrar. Add the staging NS record to the parent zone.
6. [Set up the GitHub environment](setup.md#6-set-up-the-github-environment):
   create it and set its variables.
7. [Run the first deploy](setup.md#7-run-the-first-deploy): push to the
   staging branch, or push a `deploy-*` tag. This applies the edge.
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
steps of a foundation change and of a zone change.

## Related docs

- [Architecture](../docs/architecture.md): the components and the download
  site.
- [Key custody](../docs/protocol/security.md#key-custody): how ciproxy keeps
  the Boat keys.
- [ciproxy-control-plane](../crates/ciproxy-control-plane/README.md): the
  CLI skeleton that a runtime will replace.
