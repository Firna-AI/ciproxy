# Terraform promotion and recovery

After one-time seed creation and bootstrap, each trusted deployment plans
all four roots in its own GCP project. The plan identity reads resources
and state. Its only object writes acquire and release the four exact state
locks. The workload identity has its existing roles and no state writes.
Only `deploy-apply.yml` can use the separate owner-equivalent apply account.

Seed, bootstrap, and foundation plans pass policy before their apply job
starts. The reusable workflow applies them in that order. Workloads then
deploy Helm and static files, and wait for every configured NEG zone.
The plan identity saves and checks edge. The reusable workflow applies
that binary and invalidates CDN. No apply command creates a new plan.

The artifact contains `release.tfplan`, `plan.txt`, `plan.json`, the inputs,
and metadata for each root. Metadata binds the binary digest, full
Terraform digest, checked-out commit, environment, root, workflow run, and
run attempt. Apply derives policy JSON again from the saved binary.
Editing the artifact JSON does not change what policy checks.

Main preview accepts pushes to the primary branch and `main`. Production
accepts only a successful main preview push run on `main` from this
repository. It deploys that run's `head_sha`, even if the default branch
moves meanwhile. GitHub runs `workflow_run` from the default branch, so
production also requires the default branch to be `main`.
The repository variable `PRODUCTION_DEPLOY_ENABLED` keeps production off
until launch. It must not be an environment-only variable.
See the [setup variables](setup.md#6-set-up-the-github-environment).

## Approve a destructive change

Creates, reads, no-ops, and in-place updates need no approval file.
Every delete, replacement, and forget needs one exact committed document
for that root. Both replacement orders count as destructive.

The format follows `mokly-cloud` infrastructure promotion. Its only fields
are `schemaVersion`, `environment`, `configurationDigest`, `reason`, and
`actions`. The `environment` namespace contains the scope and root, such
as `ciproxy/main-preview/edge`. The reason contains 10 to 500 characters
after trimming its ends. Actions name every destructive address and its
ordered action array. Deposed objects also name their opaque `deposed`
key. Missing, extra, duplicate, or changed entries fail.

The digest covers paths and bytes of every `.tf`, `.tf.json`, and
`.terraform.lock.hcl` under `deploy/terraform` and `deploy/environments`.
Use the committed locks. All promotion inits use `-lockfile=readonly`.
Changing any covered file requires a new approval. Approval files live at
`deploy/approvals/<env>/<root>.json`. CI requires their bytes to match
`HEAD`. Unknown JSON fields, formats, and actions fail closed.

Generate an approval from a real plan under local operator credentials.
For example, after editing an edge resource:

```sh
python3 deploy/scripts/terraform.py plan "$DEPLOY_ENV" edge
# A policy refusal keeps plan evidence for review. Review plan.txt first.
mkdir -p "deploy/approvals/$DEPLOY_ENV"
python3 deploy/scripts/terraform_policy.py approval "$DEPLOY_ENV" edge \
  ".context/plans/$DEPLOY_ENV/edge/plan.json" \
  --reason='Remove the retired download record.' \
  > "deploy/approvals/$DEPLOY_ENV/edge.json"
```

Review the document and commit it with the Terraform change. CI creates
new plans from that merged revision. The action set must still match.
Plan evidence uploads even when policy refuses a plan.

After a successful destructive apply, remove its approval in the next
merge. A remaining approval fails when its destructive actions disappear.
This prevents reuse of an old approval. If configuration or actions
change, generate and review a replacement approval before merging again.

## Retries and stale locks

A failed plan or apply needs a new plan. Rerun **all jobs**, or start a new
trusted push run. Rerunning only failed jobs cannot reuse a successful
plan from an earlier attempt. Terraform also refuses a saved plan when
its state changed. Do not copy a plan between runs or environments.

The environment workflows serialize deployment and never cancel an active
run. Never release a lock while its operation can still write state.
When a run stops unexpectedly, inspect its run URL, process, lock ID, and
backend before recovery. Confirm that the owning operation has ended.
Then an operator can select the matching remote backend and unlock it:

```sh
ROOT=foundation
terraform -chdir="deploy/environments/$DEPLOY_ENV" init -reconfigure \
  -backend-config="bucket=ciproxy-$DEPLOY_ENV-tfstate-$PROJECT_ID" \
  -backend-config="prefix=$DEPLOY_ENV/$ROOT"
terraform -chdir="deploy/environments/$DEPLOY_ENV" force-unlock LOCK_ID
```

Use `deploy/terraform/seed`, `deploy/terraform/bootstrap`, or the
environment's `edge/` directory for those roots. Check each prefix before
unlocking. Then create new plans. Do not apply the stranded binary.

## Operator recovery

CI never deletes, replaces, or forgets its state bucket or bootstrap
trust resources. The trust set contains the federation pool and providers,
CI accounts, their federation bindings, plan and apply roles and project
grants, and state-bucket grants. A committed approval cannot bypass this
rule. Use local operator credentials for such a change.

Routine in-place trust updates remain automatic. An incorrect update can
still remove CI access. A project IAM admin can restore that access with
local credentials. This includes a disabled pool, a wrong workflow claim,
or a changed binding. Inspect the real IAM and Terraform state first.
Do not disable policy or widen the workload account to repair a failure.

Under the operator account, select the environment and create a new plan:

```sh
gcloud auth application-default login
gcloud auth application-default set-quota-project "$PROJECT_ID"
python3 deploy/scripts/terraform.py plan "$DEPLOY_ENV" bootstrap
```

For a destructive recovery, generate the exact approval as above, using
`bootstrap` in place of `edge`. Outside GitHub Actions the approval can be
uncommitted. Review the binary's human plan and then apply it:

```sh
python3 deploy/scripts/terraform.py apply "$DEPLOY_ENV" bootstrap
```

After recovery, remove an unused local approval and start CI with new
plans. Do not restore the first local seed backup over current remote
state. Its backup is evidence of the initial migration only.

Cloud SQL has both Terraform provider and API deletion protection.
Secret Manager IDs have Terraform provider deletion protection. That
protection does not block deletion through the console. The apply account
can change project IAM and is owner-equivalent. The separate production
deny-policy follow-up remains necessary for wider deletion protection.
No GCP or GitHub ruleset mutation runs during offline checks.
