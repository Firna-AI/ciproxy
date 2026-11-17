# Organization linking

This guide gives the dashboard routes that link a GitHub organization to a
workspace, and the local activation of a linked organization. The
[crate README](README.md) gives the rest of the dashboard API.

GET `/session` returns `/install` as a stable dashboard navigation link.
It creates no installation attempt. POST `/installation-links/start` saves
an attempt and returns the configured GitHub URL with its opaque state.
The normal Origin and CSRF checks protect this start action. It keeps the
session's exact Direct or Add continuation purpose. An Add continues as
Direct after its installation links. A personal-account retry stays Add.
Attempts have no age limit. Add evidence is durable before GitHub opens.

POST `/workspaces/{workspace_id}/organizations/add` returns the configured
`/installations/new?state=...` URL. GitHub still chooses the organization.
The registered setup return starts the same browser-bound OAuth proof.
`setup_action=request` stays waiting and chooses no workspace.

GET `/installation-links` refreshes the original person's installed claims.
Add requires the exact signed requester and fresh own-user installation proof.
Signed Direct claims retry without Add proof, even when the person's
installation list omits them. A provisional Direct return still waits for the
signed event.
The store joins signed identity before the access service resolves admission.
Another person's provisional return supplies no flow, proof, or destination.
One current Admin workspace links Add automatically. Several require
PUT `/installation-links/{claim_id}/workspace` with an offered workspace ID.
No eligible workspace leaves the claim waiting. Exact offers survive the
choice-to-link role-loss window. Missing state never proves a Direct flow.

A linked organization has local activation separate from GitHub App state.
Billing and Admin reasons can coexist. Every linked inactive organization
counts against the tier, excluding Removed. Existing admitted organizations
stay admitted on downgrade. A current Admin can approve with POST
`/workspaces/{workspace_id}/organizations/{organization_id}/enable`.
Only a new workspace receives a first-Admin grant. An inactive link grants
no role. Retained reinstall preserves holds and completed history.
An admitted retained organization prepares its current bindings before the
registration publishes Active, including a reinstall through Add.

The session's `can_claim_admin` capability is false for an existing Admin.
Other owners retain the recovery rules of the
[security protocol](../../docs/protocol/security.md#who-gets-a-role).

The API returns exact-workspace upgrade actions only to Billing and Admin.
Other people get the reason and an ask-Admin capability. No held organization
can acquire work, register a JIT runner, or reserve/start overflow.
Run the existing maintenance method to reconcile billing eligibility and
retry saved choices. It discovers durable pending work after restart.

Pass the existing GitHub service to `DashboardAccess::finish_login` for this
operation. Organization linking needs no other constructor or ApiState field.
Fresh proof resolves before the identity lease. No browser-only grant is
retained.
