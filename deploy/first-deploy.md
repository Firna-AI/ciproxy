# First data creation and workload retry

The operator creates trust, the shared core, and namespace UID bindings.
CI creates each environment's data resources. The first workload attempt
then stops at the missing release-token version. This guide gives run
inspection and retries. Do not apply preview data or create secrets/zones
manually.

## Start the first preview run

Finish setup through [GitHub variables](setup.md#4-set-github-variables).
Use `DEPLOY_ENV=main-preview`. Keep production and runtime readiness off,
uptime false, and the CDN signing-key name unset. The registry, mirrored
release digest, namespace/UID, and all twelve identity variables must exist.

Push the reviewed deployment code to `calummoore/ci-proxy-load-balancer`,
or protected `main` after its merge. Select that push run:

```sh
DEPLOY_BRANCH=calummoore/ci-proxy-load-balancer # use main after merge
DEPLOY_SHA=FULL_COMMIT_SHA_OF_CURRENT_DEPLOYMENT_CODE
gh run list --repo=Firna-AI/ciproxy --workflow=deploy-main-preview.yml \
  --branch="$DEPLOY_BRANCH" --event=push --commit="$DEPLOY_SHA" \
  --json databaseId,headSha,event,status,conclusion
RUN_ID=SELECTED_RUN_ID
```

If a push run of this exact code already exists, start a fresh attempt:

```sh
gh run rerun "$RUN_ID" --repo=Firna-AI/ciproxy
```

Use a run that contains the current deployment code, not an older workflow.
This workflow accepts push events only. There is no `workflow_dispatch`.

## Confirm the data boundary

After the attempt finishes, inspect its jobs and failed workload log:

```sh
gh run view "$RUN_ID" --repo=Firna-AI/ciproxy \
  --json headSha,event,headBranch,attempt,conclusion,jobs \
  --jq '{headSha,event,headBranch,attempt,conclusion,jobs:[.jobs[]|{name,conclusion}]}'
gh run view "$RUN_ID" --repo=Firna-AI/ciproxy --log-failed
```

Check the selected commit/event and the nested infrastructure jobs.
`apply-data` and the infrastructure `required` aggregate must succeed.
For first preview on the primary branch, empty trust scope succeeds too.
The workload step **Deploy own namespace and static buckets, then wait for
NEGs** must reach the missing `latest` read for
`ciproxy-main-preview-release-github-token`, before Helm release installation
or upgrade.

Digest validation, data output reads, cluster credentials, and namespace
access precede that read. Infrastructure, permission, identity, digest,
export, or namespace failures are not the expected missing-version stop.
Resolve them and rerun all jobs. Do not assume an access denial means the
token is simply absent. Fresh IAM grants can need propagation.

Confirm the actual parent and version metadata with operator credentials:

```sh
gcloud secrets describe "ciproxy-$DEPLOY_ENV-release-github-token" \
  --project=ciproxy --format='value(name)'
gcloud secrets versions list "ciproxy-$DEPLOY_ENV-release-github-token" \
  --project=ciproxy --format='table(name,state)'
```

The parent must exist. A fresh setup lists no versions. These commands do
not read a secret payload. If versions already exist, this is not the fresh
missing-token case. Inspect the actual failure before proceeding.

For main preview, confirm all three zones that the data apply created:

```sh
for zone in ciproxy-main-preview ciproxy-main-preview-dashboard \
  ciproxy-main-preview-releases; do
  gcloud dns managed-zones describe "$zone" --project=ciproxy \
    --format='yaml(name,dnsName,nameServers)'
done
```

The data root also created SQL, keys, application buckets, and other secret
parents. The missing-token stop occurs before Helm, publishing, snapshot
promotion, or edge creation. No edge IP or certificate exists yet.
Return to [token storage](setup.md#6-store-the-release-token), then
[preview delegation](setup.md#7-delegate-preview-dns).

## Retry all jobs and create the edge

After token storage and delegation, rerun every job:

```sh
gh run rerun "$RUN_ID" --repo=Firna-AI/ciproxy
gh run view "$RUN_ID" --repo=Firna-AI/ciproxy \
  --json headSha,attempt,status,conclusion,jobs
```

Inspect that attempt after it completes. Workloads, snapshot plan/apply,
edge plan/apply, and invalidation must succeed. The edge now has its
address, certificate, URL map, and backend buckets. Certificate/domain
status can still be provisioning. Continue with [CDN key registration](setup.md#9-register-the-cdn-signing-key)
and [HTTPS/uptime checks](setup.md#10-enable-uptime-alerts).

Use the all-jobs form above. Do not add `--failed` or `--job`. Saved plans
and workload artifacts must bind to the new attempt. Never copy an older
binary into it. If evidence has expired, start a new accepted push run of
the current code. [GitHub rerun options](https://cli.github.com/manual/gh_run_rerun)

The first private-origin reader grant happens during edge apply. If the
organization sharing rule refuses it, stop that path and return item 76
with its existing alternatives. Do not widen policy. See
[private-origin permissions](identities.md#private-static-origins).
Other failures follow [promotion recovery](promotion.md).

## First production data run

Follow [launch](launch.md) through retirement, its negative proof,
production UID capture, environment variables, and switch enablement.
Production follows a successful preview push of the same current `main`
commit. Its first infrastructure job creates production data, then its
workload job stops at the missing production release-token version.

Select the production run triggered by that successful main preview:

```sh
DEPLOY_ENV=production
gh run list --repo=Firna-AI/ciproxy --workflow=deploy-production.yml \
  --event=workflow_run --json databaseId,headSha,event,status,conclusion
RUN_ID=SELECTED_PRODUCTION_RUN_ID
gh run view "$RUN_ID" --repo=Firna-AI/ciproxy --web
```

Check its successful upstream preview and the source SHA printed by
**Check the trusted deployment event**. A `workflow_run` run's own
`headSha` can describe the default-branch revision. Production uses the
upstream preview source, as [promotion](promotion.md) requires.

Confirm successful data/aggregate jobs and the missing-version stop for
`ciproxy-production-release-github-token`. Use the same secret metadata
commands with `DEPLOY_ENV=production`. Confirm production's two zones:

```sh
for zone in ciproxy-production-dashboard ciproxy-production-releases; do
  gcloud dns managed-zones describe "$zone" --project=ciproxy \
    --format='yaml(name,dnsName,nameServers)'
done
```

Store its token version. Inspect [production parents and child NS records](dns.md#production-parents-and-preview-children)
before delegating `app` and `get` on Cloudflare. Rerun all jobs of the
production run. Only after edge succeeds, read its IP and add the
[DNS-only apex A record](dns.md#apex-site-record). Register the production
key, check production hosts, then enable its uptime checks.
