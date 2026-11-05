# Production launch and trust handoff

Run this once after main preview's full [setup](setup.md) and successful
edge deployment. Use the installed tools and operator login from
[project preparation](setup.md#1-prepare-the-project). The operation changes
GCP trust before any production data resource exists. Later applies stay
automatic.

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

## Capture production runtime identity

After the negative revocation proof passes, install production's namespace
and operator-owned `control-plane` account on the existing shared cluster.
Capture its UID and record it in protected bootstrap state before enabling
production. Run [production UID setup](runtime-identity.md#production-identity)
with the operator credentials. That second bootstrap preserves retirement.
No production data or workload resource needs to exist for this step.

## Permit production

1. Set `DEPLOY_ENV=production`. Create its GitHub environment as
   [variable setup](setup.md#4-set-github-variables) describes.
2. Set its twelve identity variables from its selected bootstrap output.
3. Set its own `ALERT_EMAIL` and mirrored image digest. Reuse the shared
   image from preview, or mirror the current release first. Keep
   `CONTROL_PLANE_RUNTIME_READY=false` and `PUBLIC_UPTIME_ENABLED=false`.
   Leave `CDN_SIGNING_KEY_NAME` unset until production key registration.
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

Production managed state has separate `production/data` and
`production/edge` prefixes. Neither prelaunch shared nor any environment
snapshot writer can populate them. Their first plans create resources.
They never import/adopt objects from shared or preview state.

A prelaunch-created production name must stop its first create. Do not
store production secrets, publish production job images, grant database
schema rights, or enable the runtime until the data root applies cleanly.
Use [unexpected-name recovery](state.md#unexpected-existing-production-name)
under operator credentials. This adds no inventory gate.

The shared registry is reused. Trusted bootstrap grants only the shared
node account reader access at project scope in this one-registry project.
Prelaunch registry lifecycle and shared networking have no IAM setters.
Retirement therefore leaves no registry grant that unreviewed code could
install on another account.

The first production run creates its own data, then stops at the missing
release-token version before Helm. Follow [production first deploy](first-deploy.md#first-production-data-run)
to select that run and confirm infrastructure/data success, the real secret
parent, and both zones. An unrelated failure is not this expected stop.
The image, GitHub variables, and UID-bound namespace already exist.
Reapply the boundary only when its operator manifest changes:

```sh
python3 deploy/scripts/namespace_access.py production
```

This operator command also reapplies the Baseline enforce/audit/warn labels
on an existing production namespace. Reapply preview too when upgrading.
Environment deploy identities cannot change those labels.

After confirmation, [store the production token](setup.md#6-store-the-release-token).
Inspect [production DNS parents/child NS sets](dns.md#production-parents-and-preview-children),
then delegate app/get from Cloudflare. Keep Workspace mail, registrar NS,
DNSSEC, and preview NS sets unchanged. Rerun **all jobs** of that production
run. This retry creates its edge. If organization policy refuses its first
private-origin grant, stop and follow [the existing item 76 rule](identities.md#private-static-origins).
No policy exception or whole-domain zone is added.

Only after edge apply succeeds, add the [DNS-only apex A record](dns.md#apex-site-record)
from production's `public_ip`. Then [register its CDN key](setup.md#9-register-the-cdn-signing-key)
and check production's [HTTPS hosts](setup.md#10-enable-uptime-alerts) before
enabling uptime. Production finishes its own edge
and invalidation before a separate preview data/edge promotion starts.
That later promotion uses the exact successful upstream workload artifact.
Preview-only NEG failure can fail that stage without undoing production.

No migration is needed. No previous deployment has applied in either
project. Spike/test machines stay in `internal-498318`.
