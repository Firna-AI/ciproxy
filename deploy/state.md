# Deployment state ownership

The project has one shared platform and separate environment data/edges.
Root ownership does not combine customer data. No root has applied yet,
so this layout needs no migration, import, or state adoption.

| Root | Bucket | Prefix | CI writer |
| --- | --- | --- | --- |
| Seed | `ciproxy-trust-tfstate-ciproxy` | `trust/seed` | Phase-selected main-only trust |
| Bootstrap | `ciproxy-trust-tfstate-ciproxy` | `trust/bootstrap` | Phase-selected main-only trust |
| Core | `ciproxy-shared-tfstate-ciproxy` | `shared/shared` | Prelaunch shared, then production shared |
| Preview data | `ciproxy-main-preview-tfstate-ciproxy` | `main-preview/data` | Prelaunch shared, then production shared |
| Preview edge | `ciproxy-main-preview-tfstate-ciproxy` | `main-preview/edge` | Prelaunch shared, then production shared |
| Preview record | `ciproxy-main-preview-tfstate-ciproxy` | `main-preview/foundation` | Preview environment apply only |
| Production data | `ciproxy-production-tfstate-ciproxy` | `production/data` | Production shared only |
| Production edge | `ciproxy-production-tfstate-ciproxy` | `production/edge` | Production shared only |
| Production record | `ciproxy-production-tfstate-ciproxy` | `production/foundation` | Production environment apply only |

Object write conditions name these prefixes. Sharing a bucket does not
permit record writers to edit managed state. Object reads have their own
prefix conditions. Workspace discovery can list object names through a
separate role without reading other state payloads. Plan writes name
exact `default.tflock` objects. Prelaunch has no production lock grant.

The central trust accounts remain owner-equivalent through project IAM.
They can maintain state access on protected `main`. Unreviewed preview
workflows cannot impersonate them. These are trusted bootstrap authorities,
not environment record writers or prelaunch shared apply.

Production never reads data or edge resource IDs from prelaunch-writable
state. Its first managed plans create objects in empty protected states.
The shared network, NAT, cluster, registry, and runtime federation remain
in core. The registry's node reader grant belongs to trusted bootstrap.
Prelaunch has no shared registry/network IAM setter that can install a
persistent grant on another live account.

## Unexpected existing production name

A prelaunch caller can still create some names through project-level APIs.
The pinned provider does not import those objects on create. A collision
must fail the production data/edge apply. Do not populate production secret
versions, database schema, job images, or runtime before a clean data apply.

Use this operator procedure after a collision:

1. Keep runtime and data setup off. Retain the failed plan and error.
2. Select the exact target backend from the table. Inspect its state and
   the named cloud object with operator credentials.
3. Do not import the unknown object or copy preview/core state into
   production. This would preserve grants, passwords, records, or CDN keys.
4. Identify the object's owner and confirm that it holds no production
   data. Remove or quarantine an untrusted mutable object as appropriate.
5. KMS key rings and keys cannot be deleted. Make a reviewed name change
   in `terraform/modules/foundation/data.tf`. Update the matching KMS IAM
   condition in `terraform/bootstrap/permissions.tf`. Keep the production
   namespace, project, and other resource names unchanged.
6. Merge that reviewed change. Start all jobs with fresh plans. Trusted
   bootstrap applies the permission change before production data creation.
7. Complete data setup only after the target data root succeeds.

A failed partial apply can have recorded objects that production itself
created. Keep that protected state and replan normally. Do not replace it
with empty state or a prelaunch backup. This procedure adds no inventory
gate or automatic deletion/import.

The pinned provider create paths reject existing named secrets, buckets,
key rings, keys, zones, and SQL instances. This is a source-based property,
not a live authorization proof from this workspace.
[Pinned provider](https://github.com/hashicorp/terraform-provider-google/tree/v7.12.0/google/services)
