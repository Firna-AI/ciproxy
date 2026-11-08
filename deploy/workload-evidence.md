# Workload evidence and central edge promotion

A completed workload run publishes `ready-inputs.json` as the immutable
artifact `workload-<environment>-<attempt>`. It records the environment,
checkout commit, workflow run, run attempt, deployed control-plane image,
runtime/uptime flags, alert
addresses, and prepared zones. Publication follows Helm and all NEG waits.

`workload_snapshot.py` is the one reader for record publication and edge
planning. Its own rollout must match the caller's commit, run, and attempt.
For production's later preview promotion, it selects the successful
`workflow_run.id` and `workflow_run.run_attempt` from the trusted event.
The source checkout must match that run's `head_sha`.

Production also uses this reader before its workload rollout. It selects
the control-plane digest saved after the enabled rollout of that exact
preview run and attempt. A disabled preview records an empty image.
Enabled production rejects that empty image through its runtime validation.
Disabled production can still deploy static content and the release server.
It cannot build or select a different production image. The additional
image field remains in the workload artifact. The foundation record omits
it from Terraform variables, so its existing schema stays unchanged.

The executable `ci_roots.py` emits the target environment, artifact name,
and run ID. `deploy-plan.yml` downloads that exact run/name with
`actions: read`. It never selects a latest artifact or mutable snapshot
state. An old attempt, different commit, or wrong environment is rejected
by the existing binding check.
[Cross-run artifacts](https://github.com/actions/download-artifact/tree/v4#download-artifacts-from-other-workflow-runs-or-repositories)

Caller and target remain different fields. A production caller can target
preview's data/edge only after launch. Saved plans bind both identities,
the selected evidence binding, source/run/attempt, configuration digest,
and binary digest. Plan artifact names distinguish `edge` from
`preview-edge`. Approvals name the target root, such as
`ciproxy/main-preview/edge`. The policy schema stays unchanged.

Snapshot states remain useful rollout records. Environment apply accounts
can write only their own `foundation/` prefix. Central edges never use
those mutable records as authority.

## Zones and independent edges

`terraform/modules/zones/main.tf` holds desired placement and a retained
catalog of canonical zone URLs. Authenticated evidence selects catalog
keys. The edge module builds NEG links from catalog values and canonical
environment resource names. It never puts raw zone input into a URL.

Keep a catalog entry while a previous completed rollout can still select
it. A placement change need not invalidate legitimate earlier evidence.
The root's latest desired placement does not replace its prepared zones.

Production data, edge, and invalidation finish before separate preview
data/edge promotion starts. A later unreviewed preview can remove its own
NEG and break that later stage. It cannot make production's own edge read
preview state or wait for the missing preview NEG. The overall workflow
still reports the later preview failure.

## Operator plan

Use the operator's local credentials and a clean operator checkout.
Choose a successful trusted run and its exact attempt. For preview
recovery after launch, choose the successful upstream preview run.
For production, choose its completed production workload run.

```sh
DEPLOY_ENV=production
WORKLOAD_RUN=RUN_ID
WORKLOAD_ATTEMPT=ATTEMPT_NUMBER
EVIDENCE_DIR="$(mktemp -d)"
gh run view "$WORKLOAD_RUN" --repo=Firna-AI/ciproxy \
  --json conclusion,event,headBranch,headSha
gh run download "$WORKLOAD_RUN" --repo=Firna-AI/ciproxy \
  --name="workload-$DEPLOY_ENV-$WORKLOAD_ATTEMPT" --dir="$EVIDENCE_DIR"
mkdir -p ".context/deploy/$DEPLOY_ENV"
cp "$EVIDENCE_DIR/ready-inputs.json" ".context/deploy/$DEPLOY_ENV/ready-inputs.json"
rm -rf "$EVIDENCE_DIR"
SOURCE_SHA="$(jq -r .commit ".context/deploy/$DEPLOY_ENV/ready-inputs.json")"
git switch --detach "$SOURCE_SHA"
export GITHUB_RUN_ID="$WORKLOAD_RUN" GITHUB_RUN_ATTEMPT="$WORKLOAD_ATTEMPT"
python3 deploy/scripts/terraform.py plan "$DEPLOY_ENV" edge
```

Inspect the human plan before approval/apply. Use the target environment
for this operator command. Do not copy a binary between caller/target
contexts. CI applies only through the reusable apply workflow.

Tag binding belongs to the same edge plan as its URL map. The module uses
the permanent numeric `map_id` in its parent and keeps the name for CDN
invalidation. New tag/IAM access can take time to propagate. After a
denial, wait for propagation and rerun all jobs with fresh plans. There is
no separate gcloud tag write or added retry loop.
