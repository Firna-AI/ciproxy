# Terraform promotion and recovery

The [deployment](README.md) has central trust, central infrastructure,
separate central data/edge instances, and one ready-workload record per
environment. Each root
has one remote backend. No migration is needed before the first apply.

## Sequence and identities

The operator creates seed/bootstrap, the first shared core, and the runtime
ServiceAccounts. Bootstrap can run before the cluster with an empty UID map.
The operator captures preview's real UID, then updates bootstrap and shared.
Main-preview pushes on the
primary branch can then promote shared infrastructure, but cannot change
trust. On protected `main`, CI plans seed/bootstrap with a central
read-only account and applies their exact saved plans with the separate
central trust account. Seed API enablement finishes before bootstrap apply.
Later bootstrap plans preserve the UID map from protected trust output.
Shared runtime bindings read that map directly, never rollout evidence or
environment snapshot state. Production UID capture happens after retirement.
[Runtime identity procedure](runtime-identity.md) gives first use and recovery.

Fresh setup runs CI data creation after UID setup, the mirrored image, and
GitHub variables. The first workload stops at the missing token version
before Helm. The operator confirms the parent and zones, stores the token,
delegates DNS, and reruns all jobs. That retry creates the edge. See
[first deploy](first-deploy.md). Other failures do not establish this boundary.

Every required apply job runs, even when its scope selects no roots.
Event and checkout validation happen first. The environment's read-only
plan identity reads launch status. Root selection precedes privileged
apply authentication and saved-plan download. An empty result reports a
successful no-op and calls no Terraform. Nonempty results retain all saved
binary, policy, caller/target, commit, run, and attempt checks.

The phase owner then promotes central infrastructure. It uses a separate
shared apply account with no project IAM or Kubernetes object permissions.
It applies the caller's separate data state after shared core.
Workloads read central outputs and deploy only their own namespace and
static buckets. They wait for every configured NEG zone.

A ready workload saves `ready-inputs.json`. It records the actual zones,
runtime/uptime flags, deployed control-plane image, alert addresses, source
commit, environment, workflow run, and run attempt. Production selects its
image from that exact upstream preview evidence before deploying.
Only an enabled rollout records an image after Helm and NEG readiness.
A disabled preview records an empty image. Existing runtime validation
rejects enabled production without that deployed image. Static-only
production still works.
[Image promotion and rollback](control-plane-images.md) gives the operator
steps. The next read-only plan checks those bindings. The
environment apply account writes the exact saved snapshot plan only to its
own state bucket. A failed rollout never publishes a new ready snapshot.

Central edge uses the same [provenance-aware reader](workload-evidence.md)
as record publication. Production's own edge uses its current completed
rollout artifact. Its later preview edge uses the immutable artifact of
the exact successful upstream run and attempt. Latest preview state is
never authority. Canonical names and resource IDs come from that
environment's managed data state. The trusted zone catalog supplies URL
paths. Evidence selects catalog keys, never interpolated URL text.

Production edge has separate state and execution. Its own invalidation
finishes before preview data/edge promotion starts. A later preview NEG
deletion can break preview's promotion but cannot block production's
completed edge. The overall run reports that later preview failure.

Before launch, preview owns central promotion. After
[operator cutover](launch.md), production alone owns it. Preview keeps its
namespace/static deployment and publishes its ready snapshot. Production
promotes central resources only after a successful preview push of the
same commit on protected `main`. It uses that run's `head_sha` even if the
default branch moves. The default branch must be `main`.

Required infrastructure has this chain: trust plan/apply, core plan/apply,
and data plan/apply, followed by an aggregate that requires success from
all six jobs. Workloads, snapshot plan/apply, edge plan/apply, and CDN
invalidation follow. Production then promotes preview data and edge.
There are no intentional apply-job skips in an active chain. Ordinary
failure/cancellation stops dependent work. Production-off/source gating
remains at the entry workflow.

Separate workflow concurrency groups preserve pending preview runs.
Terraform remote locks serialize central state writes. Drain all
prelaunch runs before the operator handoff. After that handoff, the two
pipelines never hold working central write credentials at the same time.

## Saved plans

Plan artifacts contain `release.tfplan`, `plan.txt`, `plan.json`, inputs,
and metadata. Metadata binds the binary digest, full Terraform digest,
checkout commit, caller environment, target environment, root, workflow
run/attempt, and selected workload evidence binding.
Every init uses the committed provider locks. Promotion uses
`-lockfile=readonly`.

Only `deploy-apply.yml` can use apply credentials. Apply rechecks the
trusted event, checkout, GCP phase, metadata, and destructive policy.
It regenerates policy JSON from the saved binary. Editing the artifact
JSON cannot hide an action. Apply never creates another plan.

CI can apply seed and bootstrap after initial operator setup. The trust
account has effective owner power because those roots administer project
IAM and custom roles. It is available only through protected `main` and
the reusable apply job. Environment accounts never receive that power.

## Destructive approval

Creates, reads, no-ops, and updates need no approval file. Every delete,
replacement, and forget needs an exact committed approval. Both
replacement orders count as destructive.

The document has only `schemaVersion`, `environment`,
`configurationDigest`, `reason`, and `actions`. Its scope names the central
owner and root, such as `ciproxy/production/edge` or
`ciproxy/trust/bootstrap`. A central approval can run under either phase
owner. Saved binaries remain bound to their original environment and run.

Each action names its address and ordered action array. Deposed objects
also name their opaque `deposed` key. Missing, additional, duplicate, or
changed entries fail. The trimmed reason needs 10 to 500 characters.
Unknown JSON fields, plan formats, and actions fail closed.

The digest includes paths and bytes of every `.tf`, `.tf.json`, and
`.terraform.lock.hcl` under `deploy/terraform` and `deploy/environments`.
A configuration change needs a new approval. CI requires exact bytes from
`HEAD`.

First [download bound workload evidence](workload-evidence.md#operator-plan).
Generate an approval under operator credentials from a real saved plan:

```sh
DEPLOY_ENV=production
python3 deploy/scripts/terraform.py plan "$DEPLOY_ENV" edge
# A policy refusal retains plan evidence. Inspect plan.txt before approval.
mkdir -p "deploy/approvals/$DEPLOY_ENV"
python3 deploy/scripts/terraform_policy.py approval "$DEPLOY_ENV" edge \
  ".context/plans/$DEPLOY_ENV/edge/plan.json" \
  --reason='Remove the retired download record.' \
  > "deploy/approvals/$DEPLOY_ENV/edge.json"
```

Commit the reviewed document with its change. CI plans again from that
revision. The destructive action set must still match.

After the apply, remove the approval in the next merge. A leftover approval
fails when its destructive actions disappear. A changed action set or
configuration needs a replacement approval.

CI never deletes, replaces, or forgets trust resources. That rule covers
seed buckets/APIs, federation, accounts, IAM grants, custom roles, and
access-control tag resources. An approval cannot bypass it. The operator
handles destructive trust recovery. In-place trust updates remain automatic.

## Retries and stale locks

After failure, rerun all jobs or start a new trusted push. Another run
attempt cannot reuse an old plan. Terraform also rejects a binary after
its state changes. Never copy a plan between runs or environments.

Never unlock a live operation. Inspect its run, process, lock ID, and state
backend first. Confirm that the writer has ended. Then use the matching
root directory, bucket, and prefix from [the backend table](README.md).
For example, an operator can recover the central edge lock:

```sh
terraform -chdir=deploy/terraform/edge init -reconfigure \
  -backend-config=bucket=ciproxy-production-tfstate-ciproxy \
  -backend-config=prefix=production/edge
terraform -chdir=deploy/terraform/edge force-unlock LOCK_ID
```

Create a new plan after recovery. Do not apply the stranded binary.

## Operator recovery

An incorrect in-place trust change can still lock CI out. The operator
uses local credentials to restore access. Inspect current IAM and state.
Do not widen an environment account or disable policy to repair trust.

```sh
gcloud auth application-default login
gcloud auth application-default set-quota-project ciproxy
python3 deploy/scripts/terraform.py plan production bootstrap
```

For destructive recovery, generate the exact `trust/bootstrap` approval as
above. Outside CI, it can be uncommitted. Inspect the human plan, then run:

```sh
python3 deploy/scripts/terraform.py apply production bootstrap
```

Keep the production phase and both prelaunch accounts disabled during
recovery. Do not restore the initial local seed backup over newer remote
state. Restore a lost state from the correct retained bucket version.
Do not replace lost state with an empty state for existing resources.

Cloud SQL has provider and API deletion protection. Secrets have provider
deletion protection. Neither protection replaces the planned production
IAM deny policy. No cloud or GitHub ruleset operation runs in offline tests.

Tag bindings are inside the edge module's exact saved plan. Their parent
uses the URL map's permanent numeric `map_id`. Invalidation still uses its
name. New tag/IAM grants can take time to propagate. A permission denial
fails the run. Wait for propagation, then rerun all jobs with fresh plans.
No separate gcloud tag mutation or automatic eight-minute retry is added.
[Tag propagation](https://docs.cloud.google.com/iam/docs/tags-access-control)

Static origins remain private. Cached asset backend creation precedes the
exact LB service-agent Object Viewer grants, once per underlying bucket.
The managed sharing rule of project `ciproxy` allows exactly these grants.
A refused grant means that the rule changed. Read it before a retry.
Private-bucket CDN caching requires `FORCE_CACHE_ALL`; explicit object
metadata alone is insufficient. Each origin has an immutable wrapper, a
short mutable wrapper, and an HTML/default wrapper. HTML retains
`USE_ORIGIN_HEADERS` and publisher
`no-store`. Asset CDN/client lifetimes and URL paths are defined in
[static delivery](static-delivery.md). Real deployments invalidate both
hosts afterward. Mutable entries expire within five minutes even if a run
fails after upload and never reaches invalidation. The
publisher still uploads assets before HTML without deleting old files.
