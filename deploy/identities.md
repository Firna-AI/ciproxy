# Deployment identities

Both environments use GCP project `ciproxy`. Central accounts manage cloud
resources. Environment accounts have separate state and workload rights.
Central ownership keeps environment SQL, keys, secrets, buckets, zones, and
edges separate.

| Account | Boundary |
| --- | --- |
| Operator | Own Google account. Initial seed, trust, namespace RBAC, production handoff, secrets, and recovery. |
| `ciproxy-<env>-plan` | Read only its snapshot prefix and list workspace object names. Create/delete only its exact `foundation/default.tflock`. Read the two prelaunch account statuses. No state writes or secret values. |
| `ciproxy-<env>-apply` | Write only its own `foundation/` snapshot prefix. Read its record and prelaunch status. No project admin or Kubernetes object permissions. |
| `ciproxy-<env>-deploy` | Publish public content to its own private static buckets. Read its own release token. Connect to the cluster. Operate only its namespace through Kubernetes RBAC. Invalidate only its tagged URL map. Read global operation completion. |
| `ciproxy-shared-plan`, `ciproxy-shared-prod-plan` | Central metadata/state reads and exact central lock writes. No state writes or secret values. No Kubernetes object reads. Separate caller trust for preview and production. Preview central plan retires at launch. |
| `ciproxy-shared-preview-apply` | Core and preview cloud resources before launch. No production state writes. No project IAM, custom-role, or federation administration. Runtime policy writes name preview only. No Kubernetes object verbs. Permanently disabled at launch. |
| `ciproxy-shared-prod-apply` | Central cloud resources after launch. Production workflow on protected `main` only. Production/preview central state writes. Broader cloud roles than prelaunch shared apply. |
| `ciproxy-trust-preview-apply` | Seed and bootstrap on protected `main` before launch. Effective owner power through IAM administration. Permanently disabled at launch. |
| `ciproxy-trust-prod-apply` | Seed and bootstrap from production on protected `main`. Effective owner power through IAM administration. |
| `ciproxy-shared-gke` | Node logging/monitoring and read access to the single runtime image registry. |
| `ciproxy-<env>-control` | Cloud SQL connection/login to its own instance. Read its own runtime secrets. Encrypt/decrypt with its own Boat key. Only `ciproxy-<env>/control-plane` can impersonate it. |

The central trust accounts can update project IAM and custom roles. They
are effectively owners even without the predefined Owner role. This is a
separate central authority. The environment accounts and the
preview-accessible shared account do not have that authority.

`terraform/bootstrap` defines all grants. [State ownership](state.md) gives
prefix-level writes and reads. Environment record accounts cannot edit
managed cloud state, including preview data and edge. GCS workspace
listing has a separate object-name-only role. Prelaunch planning locks
name only trust, core, and preview managed lock objects, never production.

Prelaunch compute, DNS, monitoring, and registry lifecycle use explicit
permissions without shared-object IAM setters. Secret/KMS/bucket policy
rights name preview only. Cloud SQL database/user APIs remain project-wide
because GCP cannot scope them. Protected production state prevents
adoption of objects they could precreate. A name collision fails creation.

Trusted bootstrap owns the shared node's `roles/artifactregistry.reader`
project grant. The project has one registry. The node identity is shared
platform infrastructure, not an environment deploy/runtime identity.
Prelaunch registry lifecycle has create/get/update/delete and no IAM
setter. It cannot install a persistent registry admin on another account.
Both phase owners have node `actAs` on one account only.

## Private static origins

Site/dashboard buckets enforce PAP. They have no `allUsers` or
`allAuthenticatedUsers` grant. The load balancer service agent
`service-PROJECT_NUMBER@https-lb.iam.gserviceaccount.com` receives only
`roles/storage.objectViewer` on these two buckets. Google creates that
identity only after the first backend bucket. The edge's IAM resources
iterate backend resources, so each grant follows creation.

Shared apply policy get/set names preview's two static buckets before
launch, and both environments' four static buckets after launch. That
conditional custom-role grant excludes image, state, and trust buckets.
Separate metadata lifecycle rights can maintain application buckets but
cannot read/write their IAM policies. Main-only trusted bootstrap retains
state-bucket policy maintenance through its own resource-bound grant.
The image/CDN signing-key grant remains an operator action.

The futex.ai domain-restricted sharing rule stays. No organization-policy
exception or public GCS grant is required by this design. If the first real
service-agent grant is refused, stop that path and return item 76 with its
existing alternatives: A project-only override, B tagged-resource policy,
or C a cluster web origin. Do not select an alternative or widen policy
automatically. That outcome needs the user's answer.
[Private origin access](https://docs.cloud.google.com/cdn/docs/setting-up-cdn-with-bucket#make_your_cloud_storage_bucket_private)

## Operator permissions

The operator needs project Owner, or equivalent permissions for the
resources and IAM in seed and bootstrap. The operator also needs
`roles/container.admin` to install the initial namespace roles through
GKE's IAM authorizer. Project ownership is not a Kubernetes RoleBinding.
An existing cluster-admin RoleBinding is an alternative. Do not give this
role to shared CI apply.

With the operator account selected, grant its cluster setup role before
running the namespace command:

```sh
OPERATOR_EMAIL="$(gcloud config get-value account)"
gcloud projects add-iam-policy-binding ciproxy \
  --member="user:$OPERATOR_EMAIL" --role=roles/container.admin
```

Shared infrastructure installs runtime federation after cluster creation.
Prelaunch shared apply has policy-write access only on preview's runtime
account. Production shared apply has that resource-bound access on both.
Neither has policy-write access on a central trust account.

The operator sets enforce, audit, and warn to Baseline on both namespaces.
The labels reject `hostPath` pods. Environment deploy IAM has only cluster
get/connect, not namespace update. Its namespace Role grants no namespace
or RBAC changes. It cannot remove the operator's labels. Reapply existing
namespaces as [setup](setup.md#2-bootstrap) describes.

The namespace command creates a `Role` and `RoleBinding` in exactly one
namespace. It grants no ClusterRoleBinding. CI cannot change those grants.
The network policy permits traffic inside the same namespace, GFE probes,
external services, and cluster DNS. It blocks traffic to other workload
pods in the shared pod range. See [namespace setup](setup.md#2-bootstrap).

The Cloudflare zone operator delegates app/get and adds the apex A record.
Registrar name servers and Workspace mail stay unchanged. A repository admin
creates GitHub environments and variables. A separate image publisher can
receive `roles/storage.objectCreator` on one job-image bucket. The static
deploy account cannot publish job images.

## Baseline admission

The operator Namespace manifest sets enforce, audit, and warn to Baseline.
Baseline rejects `hostPath` volumes, including read-only `/var/log` mounts.
Deploy roles have no namespace-label update permission in RBAC or GCP IAM.
The operator owns these labels.

Reapply the manifest for each existing namespace before resuming untrusted
deploys. Use the same operator credentials and command:

```sh
python3 deploy/scripts/namespace_access.py main-preview
python3 deploy/scripts/namespace_access.py production
kubectl get namespace ciproxy-main-preview ciproxy-production \
  -L pod-security.kubernetes.io/enforce,pod-security.kubernetes.io/audit,pod-security.kubernetes.io/warn
```

Label changes do not evict existing pods. Inspect existing workloads and
remove any noncompliant pod under operator control. Subsequent pod
admission enforces Baseline. Both charts' current pod templates have an
offline upstream Baseline evaluation in the implementation evidence.
[Namespace policy labels](https://kubernetes.io/docs/tasks/configure-pod-container/enforce-standards-namespace-labels/)

## Federation

Each access class has its own pool, provider, account, and mapped access
attribute. Bindings select that exact attribute in that exact pool.
Providers verify numeric repository and owner IDs. They also verify the
environment, event, branch, and exact caller workflow ref.

- Preview accepts `push` from the primary branch or `main`.
- The prelaunch trust provider accepts only preview `push` on `main`.
- Production accepts only its `workflow_run` caller on `main`.
- Every apply provider also requires the exact
  `deploy-apply.yml@<ref>` reusable job claim.
- Apply scripts check the trusted event, source checkout, saved-plan
  commit, run, attempt, configuration digest, and binary digest.

The primary preview branch can never impersonate a trust account.
After [launch](launch.md), both prelaunch apply accounts and the preview
central plan account stay disabled. Their pools and providers stay disabled.
Switching the repository variable back to false does not restore preview
central writes.

## GCP scope limits

Cloud SQL instance conditions can limit connection and instance actions.
They cannot limit database or user management operations. Those resources
therefore have central ownership. Runtime grants still name each exact
instance. [Cloud SQL IAM Conditions](https://docs.cloud.google.com/sql/docs/postgres/iam-conditions#limitations)

Monitoring supports tag-based editing of alert policies and dashboards.
Uptime configurations and notification channels have no equivalent
resource-name/tag boundary. Central edge owns separate objects for both
environments. Environment deploy has no Monitoring write role.
[Monitoring access](https://docs.cloud.google.com/monitoring/access-control),
[Monitoring tags](https://docs.cloud.google.com/monitoring/docs/access-control-with-tags)

Compute resource-name conditions do not cover every load-balancer part.
Tags cover many additional parts. Tag-on-create support also differs by
resource. Central edge owns creation and mutation. The only environment
Compute mutation is URL-map invalidation, restricted to its centrally
installed tag. A separate unconditioned `compute.globalOperations.get`
grant lets gcloud wait for completion. An operation has no URL-map tag.
[Resource attributes](https://docs.cloud.google.com/iam/docs/conditions-resource-attributes),
[Load-balancer tags](https://docs.cloud.google.com/load-balancing/docs/tags)

GKE DNS access needs `container.clusters.connect`. Credential discovery
needs `container.clusters.get`. The custom connection role has only these
two permissions. Kubernetes RBAC supplies namespace permissions. GKE can
authorize through either IAM or RBAC, so `roles/container.developer` would
bypass the namespace boundary and is not granted.
[DNS endpoint access](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/latest/network-isolation#define_access_to_the_dns-based_endpoint),
[GKE authorization](https://docs.cloud.google.com/kubernetes-engine/docs/how-to/role-based-access-control#interaction_with_identity_and_access_management)

A narrow custom role cannot safely replace central trust power while
retaining automatic custom-role and IAM changes. Google warns that an IAM
grant administrator who can also edit those custom roles can elevate its
own access. Keep that authority in the protected-main trust account.
[IAM grant limits](https://docs.cloud.google.com/iam/docs/setting-limits-on-granting-roles)

Secret values never enter Terraform, GitHub variables, or command lines.
Supply them at hidden prompts or through private files outside Git.
Offline mocks validate configuration and policy. They do not prove real
cloud authorization. The first operator setup must verify that boundary.
