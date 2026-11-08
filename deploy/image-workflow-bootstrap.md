# Image workflow trust

Two cloud identities serve the control-plane image jobs:
`ciproxy-image-lookup` and `ciproxy-image-publish`. GitHub jobs get them
through workload identity federation. Each identity has one provider,
`github`, in the workload identity pool of the same name. Each provider
admits only the jobs of one reusable workflow, through its
`job_workflow_ref` claim. Until launch, that workflow is on the primary
branch `calummoore/ci-proxy-load-balancer`. Both providers require this
exact claim:

```text
Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@refs/heads/calummoore/ci-proxy-load-balancer
```

The preview caller calls the workflow on that branch, also when the caller
runs on `main`. Each provider maps `attribute.job_workflow_ref` from the
signed GitHub token. An ordinary job, such as the npm workload job, has no
such claim. A call of another workflow file or another branch has a
different claim. The binding changes no role permission.
[GitHub OIDC claims][oidc-claims] define `job_workflow_ref` and
`job_workflow_sha`. [Build and reuse](control-plane-images.md#build-and-reuse)
gives the job boundaries.

The primary branch has no ruleset. A push to it can change this workflow
and the tools that the workflow checks out. So this binding is a prelaunch
trust root, not protected `main`. Keep production off until
[the move to `main`](#move-the-binding-to-main-at-launch). Preview needs
no change on `main`.

## Activate the primary binding

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

The launch merge puts the complete tree on protected `main`. Until the
move, preview runs on `main` still call the primary workflow. Production
promotes the images that this workflow publishes. So move the binding
before you enable production:

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

   Before the launch handoff, the trust jobs of the `main` preview run can
   apply the same plan first. The operator plan then shows no provider
   change.

3. Repeat the provider check of [activation](#activate-the-primary-binding)
   with this claim:

   ```sh
   CLAIM=Firna-AI/ciproxy/.github/workflows/deploy-control-plane-image.yml@refs/heads/main
   ```

4. Rerun all jobs of a preview push on `main`. A preview run that still
   calls the primary workflow now fails image authentication.

Enable production only after these steps. The [launch guide](launch.md)
gives the separate, irreversible trust handoff.

[oidc-claims]: https://docs.github.com/en/actions/reference/security/oidc
