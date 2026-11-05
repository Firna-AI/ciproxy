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

Read their name servers from the central root after CI created them:

```sh
terraform -chdir=deploy/terraform/data init -reconfigure \
  -backend-config=bucket=ciproxy-main-preview-tfstate-ciproxy \
  -backend-config=prefix=main-preview/data
terraform -chdir=deploy/terraform/data output -json dns_name_servers
gcloud dns managed-zones list --project=ciproxy --filter='dnsName=ciproxy.dev.' \
  --format='table(name,nameServers)'
dig +short NS ciproxy.dev
PARENT_ZONE=ACTIVE_ZONE_ID
```

Choose the active parent whose name servers match `dig`. Use each preview
zone's four servers in the matching command:

```sh
gcloud dns record-sets create preview.ciproxy.dev. --project=ciproxy \
  --zone="$PARENT_ZONE" --type=NS --ttl=3600 \
  --rrdatas=SITE_NS1,SITE_NS2,SITE_NS3,SITE_NS4
gcloud dns record-sets create preview.app.ciproxy.dev. --project=ciproxy \
  --zone="$PARENT_ZONE" --type=NS --ttl=3600 \
  --rrdatas=APP_NS1,APP_NS2,APP_NS3,APP_NS4
gcloud dns record-sets create preview.get.ciproxy.dev. --project=ciproxy \
  --zone="$PARENT_ZONE" --type=NS --ttl=3600 \
  --rrdatas=GET_NS1,GET_NS2,GET_NS3,GET_NS4
```

At production setup, copy all required MX, TXT, and preview NS records
into `ciproxy-production`. Keep preview delegation unchanged. The new zone
has no DNSSEC. Remove any DS record before changing registrar name servers.
For production, reconfigure the same data root to bucket
`ciproxy-production-tfstate-ciproxy` and prefix `production/data`.
Its `dns_name_servers` output belongs only to production.
