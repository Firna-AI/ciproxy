# Production launch and trust handoff

Run this once after [main preview setup](setup.md). Use the operator's
local Google credentials. The operation changes GCP trust before any
production data resource exists. Later applies stay automatic.

## Prepare

1. Ensure protected `main` has both deploy workflows and the ruleset.
2. Ensure the default branch is `main`.
3. Keep repository variable `PRODUCTION_DEPLOY_ENABLED=false`.
4. Stop new preview pushes. Drain all running and queued prelaunch jobs.
5. Back up current remote state through the operator's normal process.
6. Select project `ciproxy` with application default credentials.

Do not overlap operator cutover with a preview apply. The workflows keep
separate concurrency groups so production cannot cancel a pending preview.

## Capture the credential baseline

Capture immediately before cutover. The probe requests one-hour tokens and
requires both state reads to succeed. Keep the temporary TokenCreator
grants until verification, so a removed grant cannot explain a refused
fresh token. These grants name only the two retiring accounts.

```sh
OPERATOR_EMAIL="$(gcloud config get-value account)"
PROBE_DIR="$(mktemp -d)"
for account in ciproxy-shared-preview-apply ciproxy-trust-preview-apply; do
  gcloud iam service-accounts add-iam-policy-binding \
    "$account@ciproxy.iam.gserviceaccount.com" --project=ciproxy \
    --member="user:$OPERATOR_EMAIL" --role=roles/iam.serviceAccountTokenCreator
done
python3 deploy/scripts/launch_probe.py capture "$PROBE_DIR"
```

The probe stores credentials in private files with mode 0600. It prints
only account names, expiry, and results. No bearer value enters a command
argument. Run cutover and verification before the printed expiry.

## Retire prelaunch authority

Run production's first bootstrap:

```sh
DEPLOY_ENV=production PROJECT_ID=ciproxy
NAMESPACE=ciproxy-production
gcloud auth application-default set-quota-project ciproxy
bash deploy/scripts/bootstrap.sh production --existing-state
```

The same bootstrap root now sets `production_enabled=true`. It disables
both prelaunch shared and trust service accounts, the preview central plan
account, their federation pools, and their providers. It does not create
production SQL, keys, secrets, application buckets, zones, or Kubernetes
workloads.

The script reads all three accounts back and requires `disabled=true`. A
partial handoff stops CI routing. Re-running preview bootstrap reads the
persisted phase and keeps those accounts disabled. No routine setup or CI
command re-enables them.

Disabling only a provider would leave already issued impersonated tokens.
Google states that disabling a service account rejects existing access
tokens and prevents new ones. Keep the accounts disabled permanently.
[Disable semantics](https://docs.cloud.google.com/iam/docs/reference/rest/v1/projects.serviceAccounts/disable)

Before enabling production, run the negative proof with the same operator
and capture directory. It checks `disabled=true` for both accounts. It
also requires fresh impersonation and the captured state reads to fail.
It checks token expiry before and after each negative read. An expired
token cannot produce a passing proof.

```sh
python3 deploy/scripts/launch_probe.py verify "$PROBE_DIR"
```

Keep production off until this command passes. IAM changes can take
minutes to propagate and sometimes take longer. Retry verification while
the captured tokens remain unexpired. Do not use elapsed time alone as
proof. Do not re-enable an account to repeat a probe.
[Access propagation](https://docs.cloud.google.com/iam/docs/access-change-propagation)

Remove both temporary grants and private captures after verification:

```sh
for account in ciproxy-shared-preview-apply ciproxy-trust-preview-apply; do
  gcloud iam service-accounts remove-iam-policy-binding \
    "$account@ciproxy.iam.gserviceaccount.com" --project=ciproxy \
    --member="user:$OPERATOR_EMAIL" --role=roles/iam.serviceAccountTokenCreator
done
rm -rf "$PROBE_DIR"
unset PROBE_DIR
```

If verification fails, keep the switch false. Inspect account status,
permissions, and capture expiry under operator credentials. The probe
never enables accounts or changes IAM.

## Permit production

1. Create GitHub environment `production`.
2. Set its twelve identity variables from the bootstrap output.
3. Set its own `ALERT_EMAIL`, image digest, runtime, CDN, and uptime values.
4. Limit its deployment branch to protected `main`.
5. Enable the repository switch:

   ```sh
   gh variable set PRODUCTION_DEPLOY_ENABLED --repo=Firna-AI/ciproxy --body=true
   ```

6. Start a new main preview push run on `main`, or rerun all jobs of one.
7. Let production follow that exact successful commit.

The GCP latch makes preview read central state even before the repository
switch changes. After the switch, production alone has central apply
rights. Its separate provider checks the production caller, `main`, and
the reusable apply job. Preview's disabled identities cannot use those
rights even from an edited preview workflow.

A switch enabled before the GCP handoff fails before central Terraform
calls. A switch changed back to false after launch stops production while
preview remains read-only for central infrastructure.

The first production run creates its cloud resources. It can stop at a
missing release token or namespace. Then finish setup steps 3 to 6 for
production. The registry already exists. Install its namespace boundary:

```sh
python3 deploy/scripts/namespace_access.py production
```

Store the production release token. Delegate production DNS while retaining
preview's NS records. Rerun all jobs. Register its CDN key and enable its
uptime checks only after HTTPS is ready.

No migration is needed. No previous deployment has applied in either
project. Spike/test machines stay in `internal-498318`.
