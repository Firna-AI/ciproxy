# Image workflow trust

Two cloud identities serve the control-plane image jobs:
`ciproxy-image-lookup` and `ciproxy-image-publish`. GitHub jobs get them
through workload identity federation. Each identity has one provider,
`github`, in the workload identity pool of the same name. Each provider
admits only the jobs of one reusable workflow, through its
`job_workflow_ref` claim. That workflow is on protected `main`. Both
providers require this exact claim:

```text
Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@refs/heads/main
```

The preview caller on `main` calls that workflow. A preview run on the
primary branch `calummoore/ci-proxy-load-balancer` gets the image
identities only when its own caller also calls the workflow on `main`. The
binding changes no role permission.

Each provider maps `attribute.job_workflow_ref` from the signed GitHub
token. An ordinary job, such as the npm workload job, has no such claim. A
call of another workflow file or another branch has a different claim.
[GitHub OIDC claims][oidc-claims] define `job_workflow_ref` and
`job_workflow_sha`. [Build and reuse](control-plane-images.md#build-and-reuse)
gives the job boundaries.

The prelaunch binding named the workflow on the primary branch. That branch
has no ruleset. A push to it can change that workflow and the tools that
the workflow checks out. So the prelaunch binding was a trust root outside
protected `main`.

Both providers keep the prelaunch claim until a bootstrap from protected
`main` applies [the move](#move-the-binding-to-main-at-launch). Until the
[launch retirement](launch.md#retire-prelaunch-authority), a rerun of a
`main` push from before the move can apply the prelaunch claim again. Keep
production off until the move and its launch checks pass.

## Activate the primary binding

These steps activate the prelaunch binding. After the move to `main`, do
not apply the bootstrap of a primary-branch revision. That bootstrap
applies the prelaunch claim again. Use the bootstrap and the provider check
of [the move](#move-the-binding-to-main-at-launch) instead.

For a fresh project, run that bootstrap with `--image-registry` after
shared core exists. Keep the variable setup below, and check the providers
with the claim on `main`.

Run these steps after the reviewed image workflow change merges into the
primary branch. They change nothing on `main`. They need no pull request,
merge, or ruleset there. Use the operator account and the tools of
[project preparation](setup.md#1-prepare-the-project). Complete
[initial bootstrap and shared core](runtime-identity.md) first. Shared core
creates the registry `ciproxy-shared-runtime`.

1. Check out the reviewed primary revision. `git status` must print
   nothing:

   ```sh
   PRIMARY=calummoore/ci-proxy-load-balancer
   WORKFLOW=.github/workflows/deploy-control-plane-image.yml
   git fetch origin "$PRIMARY"
   git switch --detach "origin/$PRIMARY"
   git status --short
   ```

2. Check that GitHub has the same workflow on the primary branch. Both
   commands must print the same object ID. A `404` means that preview runs
   cannot call the workflow yet:

   ```sh
   gh api "repos/Firna-AI/ciproxy/contents/$WORKFLOW?ref=$PRIMARY" --jq .sha
   git rev-parse "HEAD:$WORKFLOW"
   ```

3. Apply bootstrap with operator application default credentials:

   ```sh
   gcloud auth application-default login
   gcloud auth application-default set-quota-project ciproxy
   gcloud artifacts repositories describe ciproxy-shared-runtime \
     --location=us-central1 --project=ciproxy
   bash deploy/scripts/bootstrap.sh main-preview --existing-state --image-registry
   ```

   Review the plan before you accept it. Both image providers must end with
   the claim above. The script reads the remote bootstrap outputs. It keeps
   the launch phase and the captured runtime UIDs. So the outputs
   `production_enabled` and `runtime_service_account_uids` do not change.
   `--image-registry` sets `control_plane_registry_ready` to true. Later
   operator retries and protected-main CI plans keep that value. The
   primary preview branch cannot apply bootstrap.

4. Check both providers. The command prints `true` once for each provider:

   ```sh
   CLAIM="Firna-AI/ciproxy/$WORKFLOW@refs/heads/$PRIMARY"
   for pool in ciproxy-image-lookup ciproxy-image-publish; do
     gcloud iam workload-identity-pools providers describe github \
       --workload-identity-pool="$pool" --location=global --project=ciproxy \
       --format=json \
       | jq -e --arg condition "assertion.job_workflow_ref == '$CLAIM'" \
         '(.attributeCondition | contains($condition)) and
          .attributeMapping["attribute.job_workflow_ref"] == "assertion.job_workflow_ref"'
   done
   ```

5. Set or keep the four image variables of the `main-preview` GitHub
   environment. [Setup](setup.md#4-set-github-variables) creates that
   environment. The values come from the bootstrap output:

   ```sh
   terraform -chdir=deploy/terraform/bootstrap output -json github_environment_variables \
     | jq -r '."main-preview" | to_entries[] | select(.key | startswith("GCP_IMAGE_")) | [.key, .value] | @tsv' \
     | while IFS=$'\t' read -r name value; do
         gh variable set "$name" --repo=Firna-AI/ciproxy \
           --env=main-preview --body="$value"
       done
   ```

   Keep `CONTROL_PLANE_RUNTIME_READY` and the other runtime settings
   unchanged. Keep `CONTROL_PLANE_ROLLBACK_IMAGE` unset, unless an
   [explicit rollback](control-plane-images.md#roll-back-and-resume) is
   active. Remove a remaining `CONTROL_PLANE_IMAGE` as
   [legacy cleanup](control-plane-images.md#remove-the-legacy-image-variables)
   describes.

6. Rerun all jobs of the accepted preview push of this revision:

   ```sh
   gh run list --repo=Firna-AI/ciproxy --workflow=deploy-main-preview.yml \
     --branch="$PRIMARY" --event=push --commit="$(git rev-parse HEAD)" \
     --json databaseId,headSha,event,status,conclusion
   RUN_ID=SELECTED_RUN_ID
   gh run rerun "$RUN_ID" --repo=Firna-AI/ciproxy
   ```

   The merge can start this run before activation. Its image job then fails
   at authentication. The rerun starts a fresh attempt. Then
   [check the image run](control-plane-images.md#check-the-image-run).

In a fresh setup, set the variables with
[the other variables](setup.md#4-set-github-variables). The
[first preview run](setup.md#5-create-preview-data-through-ci) replaces the
rerun.

## Move the binding to main at launch

The launch merge puts the complete tree on protected `main`. Preview runs
of `main` commits from before the move call the primary workflow.
Production promotes the images that the trusted workflow publishes. So move
the binding before you enable production:

1. Merge one reviewed change into protected `main`. It switches these
   values from the primary branch to `main` together:

   - the caller reference in `deploy-main-preview.yml`
   - the expected claim inside `deploy-control-plane-image.yml`
   - the claim of both image providers in `terraform/bootstrap`
   - the offline tests that pin these values

2. Apply that reviewed bootstrap with operator application default
   credentials. The script keeps the launch phase, the runtime UIDs, and
   the registry flag:

   ```sh
   git fetch origin main
   git switch --detach origin/main
   git status --short
   bash deploy/scripts/bootstrap.sh main-preview --existing-state
   ```

   Before the launch handoff, the trust jobs of the preview run of that
   commit can apply the same plan first. The operator plan then shows no
   provider change.

3. Repeat the provider check of [activation](#activate-the-primary-binding)
   with this claim:

   ```sh
   CLAIM=Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@refs/heads/main
   ```

4. Rerun all jobs of the preview push of the current reviewed `main`
   commit. Use the commit that you checked out for the bootstrap apply:

   ```sh
   gh run list --repo=Firna-AI/ciproxy --workflow=deploy-main-preview.yml \
     --branch=main --event=push --commit="$(git rev-parse HEAD)" \
     --json databaseId,headSha,event,status,conclusion
   RUN_ID=SELECTED_RUN_ID
   gh run rerun "$RUN_ID" --repo=Firna-AI/ciproxy
   ```

   This run calls the workflow on `main`, so its image jobs test the new
   claim. Then
   [check the image run](control-plane-images.md#check-the-image-run).

   Do not rerun a `main` push from before the move. Until the
   [retirement](launch.md#retire-prelaunch-authority), its trust jobs
   apply the bootstrap of its own commit before its image job. That
   bootstrap restores the prelaunch claim. Then its image job can
   authenticate. So the run can pass without a test of the move.

   A push run on the primary branch applies no bootstrap. While both
   providers keep the claim on `main`, a run that calls the primary
   workflow fails image authentication.

Enable production only after these steps. The [launch guide](launch.md)
gives the separate, irreversible trust handoff. It repeats the provider
check after its last operator bootstrap, before the production switch.
After the retirement, no preview run can change the providers.

[oidc-claims]: https://docs.github.com/en/actions/reference/security/oidc
