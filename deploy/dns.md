# Deployment DNS setup

Use these operator steps after [central data setup](setup.md).
Both environments use project `ciproxy`. The registrar and current parent
zone owner supply the required permissions.

## Delegate DNS

The preview zones are in project `ciproxy`:

| Managed zone | DNS name |
| --- | --- |
| `ciproxy-main-preview` | `preview.ciproxy.dev` |
| `ciproxy-main-preview-dashboard` | `preview.app.ciproxy.dev` |
| `ciproxy-main-preview-releases` | `preview.get.ciproxy.dev` |

The active parent zone is in Cloudflare. A lookup on 5 November 2026 gave
`desi.ns.cloudflare.com` and `art.ns.cloudflare.com`. Cloudflare also holds
the domain's Google Workspace mail records.

Read the preview name servers from the central root after CI created them:

```sh
terraform -chdir=deploy/terraform/data init -reconfigure \
  -backend-config=bucket=ciproxy-main-preview-tfstate-ciproxy \
  -backend-config=prefix=main-preview/data
terraform -chdir=deploy/terraform/data output -json dns_name_servers
```

In the Cloudflare dashboard, select `ciproxy.dev`, then **DNS > Records**.
Add these three NS record sets. Each set has four entries, one for each
name server in that zone's Terraform output:

| Type | Name | Name-server values |
| --- | --- | --- |
| NS | `preview` | Servers for `preview.ciproxy.dev.` |
| NS | `preview.app` | Servers for `preview.app.ciproxy.dev.` |
| NS | `preview.get` | Servers for `preview.get.ciproxy.dev.` |

Keep the existing mail records and registrar name servers unchanged.
[Cloudflare delegation](https://developers.cloudflare.com/dns/manage-dns-records/how-to/subdomains-outside-cloudflare/)
describes delegating a subdomain to another provider.

At production setup, copy all required MX, TXT, and preview NS records
into `ciproxy-production`. Keep preview delegation unchanged. The new zone
has no DNSSEC. Remove any DS record before changing registrar name servers.
For production, reconfigure the same data root to bucket
`ciproxy-production-tfstate-ciproxy` and prefix `production/data`.
Its `dns_name_servers` output belongs only to production.
