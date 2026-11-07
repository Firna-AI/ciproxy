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

The control-plane runtime stays disabled until [runtime setup](runtime.md)
supplies its image, settings, and secret versions. The release server and
static exports can deploy independently.
The optional [Boat snapshot](runtime.md#boat-shared-snapshot) follows the
release-token delivery path and needs no secret version while Boat is off.

- [Setup](setup.md) gives the first operator steps.
- [First deploy](first-deploy.md) gives CI data creation, the expected token
  stop, confirmation, and the all-jobs retry that creates the edge.
- [Launch](launch.md) gives the one-time production trust handoff.
- [Identities](identities.md) gives the cloud and namespace boundaries.
- [Promotion](promotion.md) gives saved plans, approvals, and recovery.
- [Control-plane images](control-plane-images.md) gives automatic image
  publication, source reuse, promotion, operator grants, and rollback.
- [Operations](operations.md) gives images, runtime setup, and rotation.

## Environments

| Environment    | Namespace              | Hosts                                                                                               | Deploys on                                                 |
| -------------- | ---------------------- | --------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| `main-preview` | `ciproxy-main-preview` | `br-main.preview.ciproxy.dev`, `br-main.preview.app.ciproxy.dev`, `br-main.preview.get.ciproxy.dev` | Each push to `calummoore/ci-proxy-load-balancer` or `main` |
| `production`   | `ciproxy-production`   | `ciproxy.dev`, `app.ciproxy.dev`, `get.ciproxy.dev`                                                 | Successful preview of the same `main` commit, after launch |

Project `ciproxy` keeps domain-restricted sharing through the managed rule
`iam.managed.allowedPolicyMembers`, in place of the legacy rule. It allows the
futex.ai organization and, by name, the load balancer and CDN fill agents of
the project. The content is public through the edge, but its GCS origins
enforce PAP and have no anonymous grants. Only those two agents read them.

All regional resources use `us-central1`, except the Terraform state buckets
of the seed, which stay in `europe-west1`. Each environment has one global
HTTPS load balancer and certificate. HTTP redirects to HTTPS.
`ciproxy.dev` and Workspace mail stay on Cloudflare. Production Cloud DNS
owns only `app.ciproxy.dev` and `get.ciproxy.dev`, including their preview
child delegations. The operator adds a DNS-only apex A record to the
production edge IP. See [DNS setup](dns.md).

| Host and path                        | Backend                                                                              |
| ------------------------------------ | ------------------------------------------------------------------------------------ |
| Site host                            | Uncached HTML/docs and cached assets from one private site origin. Docs use `/docs`. |
| Dashboard host                       | Uncached HTML and cached assets from one private dashboard origin.                   |
| Dashboard `/api`, `/agent`, `/hooks` | Control plane. These paths fail while its runtime is off.                            |
| Dashboard `/images`                  | Private image bucket through signed Cloud CDN URLs.                                  |
| Download host                        | The `futex-ai/cli-release` server.                                                   |

Each static origin has separate HTML, mutable, and immutable wrappers. Private
asset caching requires `FORCE_CACHE_ALL`; object metadata alone does not
enable it. [Static delivery](static-delivery.md) gives paths and lifetimes.

The release server uses upstream commit
`30513bd1a6e15d8e9c1b18f2ed92d1d4dfa11786`. Its vendored chart has
[provenance](helm/cli-release-server/PROVENANCE.json). Both environments
pull from `us-central1-docker.pkg.dev/ciproxy/ciproxy-shared-runtime`.

The release server and control plane use `us-central1-a`,
`us-central1-b`, and `us-central1-c`. GKE prepares their NEGs in these
zones. The cluster uses the Regular channel and minimum version 1.36.4.
The NEG zone feature needs 1.36.2-gke.3104000 and is a GKE Preview feature.

## Ownership and promotion

The operator creates state, initial trust, the first shared core, and runtime
ServiceAccounts. Bootstrap records their real UIDs in protected trust state.
CI then promotes seed and
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
actual NEG zones. It also saves the immutable control-plane image. Preview
builds or reuses that image automatically. Production takes the digest from
its exact successful upstream preview artifact. Its read-only plan account
plans the `foundation` root.
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

| Path                                                 | Responsibility                                                                                                        |
| ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| `terraform/seed/`                                    | Four state buckets and trust APIs. Operator first, then main-only trust apply.                                        |
| `terraform/bootstrap/`                               | Accounts, federation, IAM, launch phase, and edge tags. Operator first, then main-only trust apply.                   |
| `terraform/shared/`                                  | Operator-first network, NAT, cluster, registry, and phase-selected UID runtime federation from protected trust state. |
| `terraform/data/`                                    | Separate central instances for environment SQL, KMS, secrets, buckets, and zones.                                     |
| `terraform/edge/`                                    | Separate central edge instances with certificates, CDN, DNS, alert channels, and uptime checks.                       |
| `environments/<env>/`                                | The completed workload snapshot for one environment.                                                                  |
| `terraform/modules/`                                 | Environment data resources and edge resources.                                                                        |
| `helm/`                                              | The release server and control-plane charts.                                                                          |
| `scripts/bootstrap.sh`                               | Operator seed, remote state preparation, and trust setup.                                                             |
| `scripts/namespace_access.py`                        | Operator namespace, RoleBinding, and network policy setup.                                                            |
| `scripts/shared_routing.py`, `scripts/ci_roots.py`   | GCP launch checks and executable CI root selection.                                                                   |
| `scripts/terraform.py`, `scripts/terraform_roots.py` | Saved-plan promotion and shared root/backend metadata.                                                                |
| `scripts/terraform_policy.py`                        | Exact destructive approval checks.                                                                                    |
| `scripts/workload_snapshot.py`                       | Inputs from a ready rollout, bound to its commit and run attempt.                                                     |
| `scripts/control_plane_image.py`                     | Exact source hashing, credentialless build, registry reuse/publication, and bound production image selection.         |
| `scripts/deploy.py`                                  | Own-namespace Helm, static publishing, NEG readiness, and CDN invalidation.                                           |
| `scripts/boat_secret.py`                             | Optional Boat snapshot delivery over standard input and its public rollout revision.                                  |
| `scripts/github-ruleset.sh`                          | Idempotent protected `main` ruleset setup.                                                                            |
| `org-policies/`                                      | The managed sharing rule of project `ciproxy`, applied once by an operator.                                           |
| `tests/`                                             | Executable CI tests and recorded plan-policy cases.                                                                   |
| `.github/workflows/deploy-*.yml`                     | Trusted event, plan, reusable apply, and workload ordering.                                                           |

| State bucket                           | Prefixes                                                            |
| -------------------------------------- | ------------------------------------------------------------------- |
| `ciproxy-trust-tfstate-ciproxy`        | `trust/seed`, `trust/bootstrap`                                     |
| `ciproxy-shared-tfstate-ciproxy`       | `shared/shared`                                                     |
| `ciproxy-main-preview-tfstate-ciproxy` | `main-preview/data`, `main-preview/edge`, `main-preview/foundation` |
| `ciproxy-production-tfstate-ciproxy`   | `production/data`, `production/edge`, `production/foundation`       |

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
npm --prefix ts run site:build
npm --prefix ts run app:export
helm lint deploy/helm/ciproxy
helm lint deploy/helm/cli-release-server --values deploy/helm/release-values.yaml
helm template cli-release-server deploy/helm/cli-release-server \
  --values deploy/helm/release-values.yaml
helm template ciproxy deploy/helm/ciproxy
helm template ciproxy deploy/helm/ciproxy --set controlPlane.enabled=true \
  --set controlPlane.image=example.invalid/control-plane:render-check \
  --set 'allowedZones={us-central1-a,us-central1-b,us-central1-c}' \
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
CDN secret version 1 and uses the operator-owned `control-plane` account.
Build both exports before the inventory routing test. It reads their actual
paths and the mocked Terraform URL map. The namespace command prints
manifests only.
Static publishing also supports a local smoke check:

```sh
npm --prefix ts run site:build
python3 deploy/scripts/publish_static.py --directory=ts/site/dist \
  --bucket=dry-run --dry-run
```

No offline check applies Terraform, changes IAM, or changes GitHub rules.
