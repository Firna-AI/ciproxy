# Operator runtime identity

Each runtime Google service account trusts one operator-created Kubernetes
ServiceAccount UID. Matching namespace/name in another cluster does not
match this UID. Helm uses `control-plane` and never owns that object.
Shared Terraform reads the UID map only from protected bootstrap state.
Preview artifacts and environment snapshot writers cannot change it.

The member is an individual principal, not a principal set:

```text
principal://iam.googleapis.com/projects/PROJECT_NUMBER/locations/global/workloadIdentityPools/ciproxy.svc.id.goog/kubernetes.serviceaccount.uid/SERVICEACCOUNT_UID
```

[Google UID identifiers](https://docs.cloud.google.com/kubernetes-engine/docs/concepts/workload-identity#principal-id-examples)
lists this form. [Workload Identity User](https://docs.cloud.google.com/iam/docs/service-account-permissions#workload-identity-user)
permits GKE workload impersonation. Their combination supports the binding.
Google's impersonation how-to illustrates only the name form. The first
live runtime setup must prove UID token exchange. If it fails, stop the
runtime path and return the finding. Never substitute name or wildcard trust.

## First shared core and preview identity

Run from the repository root with operator application-default credentials.
Finish [initial seed/bootstrap](setup.md#2-bootstrap) first. Do not start
untrusted preview runs before this procedure finishes.

1. Create the intended core with exact saved plans. The first bootstrap
   output has an empty UID map, so no runtime binding needs a cluster yet.

   ```sh
   python3 deploy/scripts/terraform.py plan main-preview shared
   cat .context/plans/main-preview/shared/plan.txt
   python3 deploy/scripts/terraform.py apply main-preview shared
   gcloud container clusters describe ciproxy-shared --region=europe-west1 \
     --project=ciproxy --format='yaml(name,location,autopilot.enabled,workloadIdentityConfig)'
   ```

   Confirm `ciproxy-shared`, `europe-west1`, Autopilot enabled, and pool
   `ciproxy.svc.id.goog`. Prelaunch CI has no cluster create/delete or node
   attachment rights. It cannot substitute a Standard cluster at this name.

2. Grant the operator namespace setup rights and create preview's account:

   ```sh
   OPERATOR_EMAIL="$(gcloud config get-value account)"
   gcloud projects add-iam-policy-binding ciproxy \
     --member="user:$OPERATOR_EMAIL" --role=roles/container.admin
   python3 deploy/scripts/namespace_access.py main-preview \
     --uid-file=.context/deploy/ciproxy/operator-runtime-uids.json
   kubectl get serviceaccount control-plane --namespace=ciproxy-main-preview \
     -o jsonpath='{.metadata.uid}{"\n"}'
   ```

   The command pins the intended project/region/cluster before applying.
   It installs Baseline labels, namespace RBAC, network policy, and the
   runtime annotation. It records the real server UID in the local file.
   Reapplication preserves an existing account's UID.

3. Record the captured UID through operator bootstrap, then install binding:

   ```sh
   bash deploy/scripts/bootstrap.sh main-preview --existing-state \
     --runtime-uids=.context/deploy/ciproxy/operator-runtime-uids.json
   python3 deploy/scripts/terraform.py plan main-preview shared
   cat .context/plans/main-preview/shared/plan.txt
   python3 deploy/scripts/terraform.py apply main-preview shared
   terraform -chdir=deploy/terraform/bootstrap output -json runtime_service_account_uids
   ```

   Compare the stored UID with the Kubernetes output. Bootstrap merges
   captured entries into its existing map. Later operator/CI bootstrap
   plans preserve that trusted map. A missing entry installs no member.
   Keep this capture outside Git. Resume setup with the [mirrored image](setup.md#3-push-the-release-image),
   [GitHub variables](setup.md#4-set-github-variables), and first CI data run.
   This procedure created no preview application secret or DNS zone.

## Production identity

Keep the repository production switch false. Finish the launch guide's
[retirement and negative credential proof](launch.md#retire-prelaunch-authority).
Then use the same operator on the existing shared cluster:

```sh
python3 deploy/scripts/namespace_access.py production \
  --uid-file=.context/deploy/ciproxy/operator-runtime-uids.json
bash deploy/scripts/bootstrap.sh production --existing-state \
  --runtime-uids=.context/deploy/ciproxy/operator-runtime-uids.json
terraform -chdir=deploy/terraform/bootstrap output -json runtime_service_account_uids
kubectl get serviceaccount control-plane --namespace=ciproxy-production \
  -o jsonpath='{.metadata.uid}{"\n"}'
```

Compare production's stored UID with its Kubernetes output. Bootstrap
preserves preview's UID even on another operator machine with a fresh file.
Production's first normal shared saved-plan promotion installs its member.
Continue [permit production](launch.md#permit-production). Populate data
and enable runtime only after their existing setup checks pass.

## ServiceAccount recreation

Do not delete the account for routine rotation or Helm updates. A namespace
or cluster recreation also creates new ServiceAccount UIDs. The old member
does not match the replacement. Deployment scripts cannot recover trust
from their own rollout evidence.

1. Pause dependent runtime deployment. Use operator credentials to inspect
   the intended cluster and account. Repair their cause before capture.
2. Reapply `namespace_access.py ENV --uid-file=...` on that cluster.
   Do not replace a healthy account just to change its UID.
3. Run `bootstrap.sh ENV --existing-state --runtime-uids=...`.
   After launch use `production`, including for preview UID recovery.
4. Generate the shared plan and its exact destructive replacement approval:

   ```sh
   PHASE_OWNER=production # use main-preview only before launch
   python3 deploy/scripts/terraform.py plan "$PHASE_OWNER" shared
   cat ".context/plans/$PHASE_OWNER/shared/plan.txt"
   mkdir -p deploy/approvals/shared
   python3 deploy/scripts/terraform_policy.py approval "$PHASE_OWNER" shared \
     ".context/plans/$PHASE_OWNER/shared/plan.json" \
     --reason='Replace the operator-verified runtime ServiceAccount UID.' \
     > deploy/approvals/shared/shared.json
   ```

   A policy refusal retains plan evidence. Review and commit the approval
   through the normal protected change. CI creates fresh bound plans and
   replaces the old member. Do not reuse a binary from another run.
5. Verify token exchange and runtime readiness. Remove the consumed approval
   in the next merge. Keep the old account absent and no name-based member.

For a cluster replacement before launch, prelaunch CI cannot create/delete
or attach nodes. Use operator recovery and the existing destructive policy.
Ordinary exact-cluster updates remain automatic. A node-account/default
change that needs `actAs` also requires operator handling before launch.
Protected production retains lifecycle and node attachment on the shared
node account. Neither phase owner gets Kubernetes object verbs from these
cluster management grants.

## First live token exchange

Before runtime enablement, run this operator probe in the selected namespace.
Use `DEPLOY_ENV=main-preview` or `production`. It prints no bearer token.
It requests a token using the operator-created account and checks that the
metadata server returns an OAuth access token. It creates no IAM grant.

```sh
kubectl run runtime-uid-probe --namespace="ciproxy-$DEPLOY_ENV" \
  --restart=Never --rm -i --attach=true \
  --image=gcr.io/google.com/cloudsdktool/google-cloud-cli:592.0.0-stable \
  --overrides='{"apiVersion":"v1","spec":{"serviceAccountName":"control-plane"}}' \
  --command -- python3 -c '
import json
import urllib.request
request = urllib.request.Request(
    "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token",
    headers={"Metadata-Flavor": "Google"},
)
with urllib.request.urlopen(request) as response:
    token = json.load(response)
assert token.get("access_token") and token.get("token_type") == "Bearer"
print("UID token exchange succeeded. Expiry seconds:", token["expires_in"])
'
```

A failure blocks runtime enablement. Inspect the captured UID, annotation,
stored binding, and IAM propagation. Do not grant a namespace/name member
as a workaround. Then complete the normal SQL and runtime checks.
