# Deployment identities

The [deployment setup](setup.md) uses these identities in each environment.

| Identity | Used for | Permissions |
| --- | --- | --- |
| Operator: your own Google account | One-time seed and trust setup, imports, secrets, and recovery | Owner, or the apply roles in `terraform/bootstrap/permissions.tf` |
| Registrar or parent zone owner | Step 5 | Change the name servers of `ciproxy.dev`, or add the three preview NS records to its active parent zone |
| GitHub repository admin | Step 6 | Create environments and their variables in `Firna-AI/ciproxy` |
| GitHub writer | Step 7 and each later deploy | Push to the primary branch, then merge a pull request into protected `main` |
| Image publisher | [Publish job images](operations.md#publish-job-images) | `roles/storage.objectCreator` on the image bucket, or the operator roles |
| Plan: `ciproxy-<env>-plan` | Save and check every root plan | Read roles and state reads. Its only object writes create and delete the four exact `<env>/<root>/default.tflock` objects. It cannot write state or read secret values. |
| Apply: `ciproxy-<env>-apply` | Apply exact saved plans through `deploy-apply.yml` | The resource admin roles and project IAM administration in `terraform/bootstrap/permissions.tf`. Project IAM administration makes this account owner-equivalent. |
| Deploy: `ciproxy-<env>-deploy` | Helm and static publishing | Existing load-balancer, network viewer, container developer, DNS, monitoring, and service consumer roles. It reads state, writes public static buckets, and reads only the release token. It has no state writes or new admin roles. |
| GKE nodes: `ciproxy-<env>-gke` | Image pulls | Read the runtime Artifact Registry repository |
| Runtime: `ciproxy-<env>-control` | The future control plane, through the Kubernetes service account `ciproxy/control-plane` | Cloud SQL login, the runtime secrets, and the Boat KMS key |

Each environment has separate plan, apply, and deploy federation providers.
The operator prepares all four remote states before CI uses these accounts.
Seed enables trust APIs before bootstrap creates accounts or permissions.
Mapped access attributes keep their service-account bindings separate.
Each condition checks the numeric repository and owner IDs, environment,
event, ref, and exact caller workflow ref:

- The repository and owner IDs are the numeric IDs of `Firna-AI/ciproxy`.
- The environment is `<env>`. The caller is
  `Firna-AI/ciproxy/.github/workflows/deploy-<env>.yml@<ref>`.
- Main preview accepts `refs/heads/calummoore/ci-proxy-load-balancer` and
  `refs/heads/main`. Its workflow triggers on the same two branches.
- Main preview needs a `push`. Production needs `workflow_run` on
  `refs/heads/main`.
- Apply also needs `job_workflow_ref` to name
  `Firna-AI/ciproxy/.github/workflows/deploy-apply.yml@<ref>`.

The condition does not use the subject claim, so the trust needs no change
to the OIDC settings of the repository.

Secret values never go into Terraform, GitHub variables, Git, or a command
line. Give each one at a hidden prompt or in a private file outside Git.
