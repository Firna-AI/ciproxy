# Deployment

This directory deploys ciproxy to the GCP project `ciproxy`. Main preview
and production share one network, NAT, GKE Autopilot cluster, and Artifact
Registry repository. Spike machines stay in `internal-498318`.

Each environment has separate Cloud SQL instances, databases, IAM users,
Boat KMS keys, secrets, buckets, DNS zones, and HTTPS edges. Each environment
has its own Kubernetes namespace and two Helm releases. Both namespaces
enforce, audit, and warn at the Baseline PodSecurity level. Only the
operator Namespace manifest manages these labels. Central Terraform
ownership does not combine their data or routing.

The control-plane runtime stays disabled until a real HTTP image exists.
The current binary is a CLI skeleton. The release server and static exports
can deploy independently.

- [Setup](setup.md) gives the first operator steps.
- [Launch](launch.md) gives the one-time production trust handoff.
- [Identities](identities.md) gives the cloud and namespace boundaries.
- [Promotion](promotion.md) gives saved plans, approvals, and recovery.
- [Operations](operations.md) gives images, runtime setup, and rotation.

## Environments

| Environment | Namespace | Hosts | Deploys on |
| --- | --- | --- | --- |
| `main-preview` | `ciproxy-main-preview` | `br-main.preview.ciproxy.dev`, `br-main.preview.app.ciproxy.dev`, `br-main.preview.get.ciproxy.dev` | Each push to `calummoore/ci-proxy-load-balancer` or `main` |
| `production` | `ciproxy-production` | `ciproxy.dev`, `app.ciproxy.dev`, `get.ciproxy.dev` | Successful preview of the same `main` commit, after launch |

The futex.ai domain-restricted sharing rule stays unchanged. The content
is public through the edge, but its GCS origins enforce PAP and have no
anonymous grants. Only the exact load balancer service agent gets origin
Object Viewer access after backend creation. No policy exception is needed.

All regional resources use `europe-west1`. Each environment has one global
HTTPS load balancer and certificate. HTTP redirects to HTTPS.
`ciproxy.dev` and Workspace mail stay on Cloudflare. Production Cloud DNS
owns only `app.ciproxy.dev` and `get.ciproxy.dev`, including their preview
child delegations. The operator adds a DNS-only apex A record to the
production edge IP. See [DNS setup](dns.md).

| Host and path | Backend |
| --- | --- |
| Site host | Private static site origin through the load balancer/Cloud CDN. Docs use `/docs`. |
| Dashboard host | Private static dashboard origin through the load balancer/Cloud CDN. |
| Dashboard `/api`, `/agent`, `/hooks` | Control plane. These paths fail while its runtime is off. |
| Dashboard `/images` | Private image bucket through signed Cloud CDN URLs. |
| Download host | The `futex-ai/cli-release` server. |

The release server uses upstream commit
`30513bd1a6e15d8e9c1b18f2ed92d1d4dfa11786`. Its vendored chart has
[provenance](helm/cli-release-server/PROVENANCE.json). Both environments
pull from `europe-west1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime`.

The release server and control plane use `europe-west1-b`,
`europe-west1-c`, and `europe-west1-d`. GKE prepares their NEGs in these
zones. The cluster uses the Regular channel and minimum version 1.36.4.
The NEG zone feature needs 1.36.2-gke.3104000 and is a GKE Preview feature.

## Ownership and promotion

The operator creates state and initial trust. CI then promotes seed and
bootstrap from protected `main`. The primary preview branch cannot use the
central trust account. Only the reusable apply workflow can use apply
accounts. See the [identity boundary](identities.md).

Before launch, preview promotes the central infrastructure and edge roots.
After the operator revokes prelaunch authority, preview reads those roots.
Production promotes them after a successful preview of the same `main`
commit. The repository switch starts production. The GCP account status
controls which pipeline can write central infrastructure. Empty scopes
finish as successful no-ops. Required apply jobs do not intentionally skip;
privileged auth, artifact download, and Terraform run only for selected roots.

Each successful workload rollout saves its own flags, alert addresses, and
actual NEG zones. Its read-only plan account plans the `foundation` root.
Its environment apply account writes that snapshot to its own `foundation/`
state prefix.
Each central edge plan uses immutable workload evidence. Production reads
its current rollout artifact. Its later preview promotion reads the exact
successful upstream preview run and attempt. Mutable snapshot state is a
record only. [Workload evidence](workload-evidence.md) gives the binding.

This small environment root is necessary. It records inputs of a ready
rollout without giving an environment cloud administration. Central data and
edge roots use separate environment state prefixes.
Prelaunch and environment record writers cannot write production managed
state. [State ownership](state.md) gives each prefix and collision handling.
Preview and production keep
separate workflow concurrency groups. Terraform locks serialize state
writes. The operator drains preview before the launch handoff.

## Layout

| Path | Responsibility |
| --- | --- |
| `terraform/seed/` | Four state buckets and trust APIs. Operator first, then main-only trust apply. |
| `terraform/bootstrap/` | Accounts, federation, IAM, launch phase, and edge tags. Operator first, then main-only trust apply. |
| `terraform/shared/` | One network, NAT, cluster, registry, and phase-selected runtime federation. |
| `terraform/data/` | Separate central instances for environment SQL, KMS, secrets, buckets, and zones. |
| `terraform/edge/` | Separate central edge instances with certificates, CDN, DNS, alert channels, and uptime checks. |
| `environments/<env>/` | The completed workload snapshot for one environment. |
| `terraform/modules/` | Environment data resources and edge resources. |
| `helm/` | The release server and control-plane charts. |
| `scripts/bootstrap.sh` | Operator seed, remote state preparation, and trust setup. |
| `scripts/namespace_access.py` | Operator namespace, RoleBinding, and network policy setup. |
| `scripts/shared_routing.py`, `scripts/ci_roots.py` | GCP launch checks and executable CI root selection. |
| `scripts/terraform.py`, `scripts/terraform_roots.py` | Saved-plan promotion and shared root/backend metadata. |
| `scripts/terraform_policy.py` | Exact destructive approval checks. |
| `scripts/workload_snapshot.py` | Inputs from a ready rollout, bound to its commit and run attempt. |
| `scripts/deploy.py` | Own-namespace Helm, static publishing, NEG readiness, and CDN invalidation. |
| `scripts/github-ruleset.sh` | Idempotent protected `main` ruleset setup. |
| `tests/` | Executable CI tests and recorded plan-policy cases. |
| `.github/workflows/deploy-*.yml` | Trusted event, plan, reusable apply, and workload ordering. |

| State bucket | Prefixes |
| --- | --- |
| `ciproxy-trust-tfstate-ciproxy` | `trust/seed`, `trust/bootstrap` |
| `ciproxy-shared-tfstate-ciproxy` | `shared/shared` |
| `ciproxy-main-preview-tfstate-ciproxy` | `main-preview/data`, `main-preview/edge`, `main-preview/foundation` |
| `ciproxy-production-tfstate-ciproxy` | `production/data`, `production/edge`, `production/foundation` |

## Offline checks

Use Terraform 1.17.1, Helm 4.3.1, actionlint, Python 3, and the pinned Node
version. These commands use no cloud credentials. Initialization downloads
only pinned provider binaries.

```sh
terraform fmt -check -recursive deploy
for root in deploy/terraform/{seed,bootstrap,shared,data,edge} \
  deploy/environments/{main-preview,production}; do
  terraform -chdir="$root" init -backend=false -input=false -lockfile=readonly
  terraform -chdir="$root" validate
done
terraform -chdir=deploy/terraform/bootstrap test
terraform -chdir=deploy/terraform/shared test
terraform -chdir=deploy/terraform/data test
terraform -chdir=deploy/terraform/edge test
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
  --set controlPlane.secrets.cdn-signing-key=projects/ciproxy/secrets/ciproxy-main-preview-cdn-signing-key
actionlint .github/workflows/deploy-*.yml
for script in deploy/scripts/*.sh; do bash -n "$script"; done
python3 -m py_compile deploy/scripts/*.py
python3 -m unittest discover -s deploy/tests -p 'test_*.py'
python3 deploy/scripts/deploy.py --help
python3 deploy/scripts/namespace_access.py main-preview --render
```

The default chart render has no control plane. The enabled render selects
CDN secret version 1. The namespace command prints manifests only.
Static publishing also supports a local smoke check:

```sh
npm --prefix ts run site:build
python3 deploy/scripts/publish_static.py --directory=ts/site/dist \
  --bucket=dry-run --dry-run
```

No offline check applies Terraform, changes IAM, or changes GitHub rules.
