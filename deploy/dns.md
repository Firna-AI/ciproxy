# Deployment DNS setup

`ciproxy.dev` stays authoritative on Cloudflare. Keep its Google Workspace
MX, TXT, SPF, and verification records. Keep registrar name servers,
DNSSEC settings, and unrelated existing records unchanged. Keep preview NS
sets after creating them. Both Google Cloud environments use project
`ciproxy`.

No Cloudflare provider or CI credential is used. The operator adds the
Cloudflare records below. Google Cloud manages the delegated zones and
both environments' HTTPS edges.

A zone on the last free Cloud DNS name-server set can need a TXT ownership
check. Complete that check before retrying the data run. See
[DNS limits](https://docs.cloud.google.com/dns/quotas#name_server_limits).

## Main preview

Wait for the first preview CI infrastructure/data apply to succeed. Confirm
the expected missing-token stop as [first deploy](first-deploy.md#confirm-the-data-boundary)
describes before reading zone outputs. Its three zones and hosts are:

| Managed zone | DNS name | Cloudflare NS name |
| --- | --- | --- |
| `ciproxy-main-preview` | `preview.ciproxy.dev` | `preview` |
| `ciproxy-main-preview-dashboard` | `preview.app.ciproxy.dev` | `preview.app` |
| `ciproxy-main-preview-releases` | `preview.get.ciproxy.dev` | `preview.get` |

Read the actual Google name servers from the preview data backend:

```sh
terraform -chdir=deploy/terraform/data init -reconfigure -lockfile=readonly \
  -backend-config=bucket=ciproxy-main-preview-tfstate-ciproxy \
  -backend-config=prefix=main-preview/data
terraform -chdir=deploy/terraform/data output -json dns_name_servers
```

In Cloudflare, select `ciproxy.dev`, then **DNS > Records**. Create each NS
set with one entry for each of its four Google name servers. The records
are DNS delegations, not proxied web records.

## Production parents and preview children

Production creates only these two Cloud DNS zones:

| Managed zone | DNS name | Cloudflare NS name |
| --- | --- | --- |
| `ciproxy-production-dashboard` | `app.ciproxy.dev` | `app` |
| `ciproxy-production-releases` | `get.ciproxy.dev` | `get` |

Wait for the first production CI data apply before changing Cloudflare.
Confirm its result through [production first deploy](first-deploy.md#first-production-data-run).
That apply also creates
these required child NS sets in the new Google parents:

| Parent zone | NS record | Name-server source |
| --- | --- | --- |
| `ciproxy-production-dashboard` | `preview.app.ciproxy.dev.` | Canonical `ciproxy-main-preview-dashboard` Google zone |
| `ciproxy-production-releases` | `preview.get.ciproxy.dev.` | Canonical `ciproxy-main-preview-releases` Google zone |

Terraform reads those named zones through `google_dns_managed_zone` data
sources. The child records use their actual `name_servers`. Rollout records
and shared state do not supply these delegations.

Read production's parent name servers and inspect the completed child
records before adding the Cloudflare `app` and `get` NS sets:

```sh
terraform -chdir=deploy/terraform/data init -reconfigure -lockfile=readonly \
  -backend-config=bucket=ciproxy-production-tfstate-ciproxy \
  -backend-config=prefix=production/data
terraform -chdir=deploy/terraform/data output -json dns_name_servers
terraform -chdir=deploy/terraform/data state show \
  'module.environment.google_dns_record_set.preview_delegation["dashboard"]'
terraform -chdir=deploy/terraform/data state show \
  'module.environment.google_dns_record_set.preview_delegation["releases"]'
```

After token storage, add NS sets named `app` and `get` in Cloudflare, with
one entry for each parent zone name server. Keep the old preview NS sets.
Once the parents are delegated, resolvers use their child NS records for
`preview.app` and `preview.get`. Cloudflare's old deeper NS sets alone
cannot preserve those children. `preview.ciproxy.dev` still delegates
directly from Cloudflare.

Cloud DNS manages production A records only for `app.ciproxy.dev` and
`get.ciproxy.dev`. It creates no whole-domain zone and no apex A record.
Do not copy mail records into Google or switch the registrar to Google.
[Cloudflare delegation](https://developers.cloudflare.com/dns/manage-dns-records/how-to/subdomains-outside-cloudflare/)
describes the parent/child authority boundary.

## Apex site record

The site host `ciproxy.dev` stays on production's Google HTTPS edge, managed
certificate, and static Cloud CDN backend. The initial token failure leaves
no edge. Store the token, delegate app/get after inspecting child NS sets,
then rerun all production jobs. Only after edge apply succeeds, read its IP:

```sh
terraform -chdir=deploy/terraform/edge init -reconfigure -lockfile=readonly \
  -backend-config=bucket=ciproxy-production-tfstate-ciproxy \
  -backend-config=prefix=production/edge
terraform -chdir=deploy/terraform/edge output -raw public_ip
```

In Cloudflare **DNS > Records**, add the site record once:

| Type | Name | Content | Proxy status |
| --- | --- | --- | --- |
| A | `@` | The production `public_ip` output | **DNS only** |

Keep it DNS only. Clients must reach Google's edge directly so Google's
managed certificate and CDN serve this host. Do not enable Cloudflare's
proxy for this A record. Keep all mail and existing preview records.
Recheck the IP only after a reviewed edge-address replacement.
[Cloudflare proxy status](https://developers.cloudflare.com/dns/proxy-status/)
explains the difference between direct and proxied DNS answers.

Verify DNS and HTTPS before enabling uptime checks. This guide describes
future operator work. No DNS or provider operation runs in offline checks.
