# Repository access and scale sets

This guide describes how the GitHub App integration keeps repository
access, runner groups, scale sets, and their workspace ownership. The
[crate README](README.md) gives the rest of the integration.

## Repository access

Repository refresh reads every installation page and saves the default
branches. The group's selected repositories intersect the customer's
choice with visible private repositories. Public access stays false.
An all-private choice includes new visible private repositories on refresh.
The runtime refreshes repository visibility at startup and every five minutes.
This needs no optional webhook subscription.
Verified `repository` and `installation_repositories` deliveries also trigger
refresh when a provider sends them.

## Runner groups

Each environment keeps one managed runner group in each GitHub
organization. An [environment](../../deploy/README.md#environments) is one
deployed control plane with its own GitHub App, such as main preview or
production. The managed group name is `ciproxy-<app id>-<organization id>`.
The App ID is the GitHub App ID of the environment, from the control-plane
setting `GITHUB_APP_ID`. The organization ID is the stable GitHub
organization ID. For example, App ID `3571` and organization ID `17` give
`ciproxy-3571-17`.

GitHub documents no length limit for a runner-group name. The
[REST docs][group-api] and the [OpenAPI description][rest-openapi] of
`POST /orgs/{org}/actions/runner-groups` give no limit. GitHub accepted the
46-character group name of the live selected-group probe. That name is
`ciproxy-probe-` and 32 hexadecimal digits. A managed name has 25
characters for a 7-digit App ID and a 9-digit organization ID.

An environment changes only the group of its saved ID or of its managed
name, and the scale sets in that group. Reconcile first reads a saved
managed group ID, including retained registrations. The saved ID finds the
group under any name. Reconcile then sets the managed name and the
repository policy on that same group. So the group keeps its ID and its
scale sets after a rename by an organization owner. The saved group ID is
the only proof that a group belongs to an environment. When two environments
saved the same group ID, they share that group until an operator splits it.

When no saved ID finds a group, reconcile looks up the group by name. That
lookup matches the full managed name only, so it does not find the group
of another environment. After local data is purged, the lookup finds a
group again only when the group still has the managed name. A group under
any other name stays on GitHub unchanged, for example a group named
`ciproxy-<organization id>` or a group that an organization owner renamed.
Reconcile creates the managed group instead. An organization owner can
delete the group with the other name when no environment uses it.

Runner group sync creates or updates the managed name and private-access
policy. It sets selected repositories through the separate repository
access endpoint after either operation. The create and update bodies omit
repository IDs, so repeated syncs use GitHub's supported update fields.

GitHub supports self-hosted runner groups on its Free plan, as its
[Free runner-group announcement][free-groups] confirms.
The live selected-group probe verified the selected-repository policy on
GitHub Free. GitHub runs the job of a selected repository on a runner of
the group. It keeps the job of another private repository queued with no
runner.

The GitHub Apps of two environments can be installed in one organization.
GitHub keeps scale-set names unique within one runner group, so the groups
of two environments can hold scale sets with the same label names. The
groups of two environments can also allow the same repository. GitHub
gives a job to either environment only when the scale sets of both
environments are eligible for the job. GitHub has no setting that controls
the choice among eligible scale sets. The
[high-availability section][arc-scale-sets] of GitHub's scale-set guide
gives both rules.

A workflow can limit a job to the runners of one group. To do this, it
sets the `group` key of `runs-on` together with `labels`. GitHub's guide
[Choosing the runner for a job][runner-group-labels] documents this rule
for self-hosted runners. GitHub's scale-set pages do not show the `group`
key. No live probe has checked this rule for ciproxy's scale sets.

## Scale sets

The adapter uses `scaleset` 0.2.0. The foundation live spike used 0.1.0.
The integration tests check the current version with the spike's wire
format and nanosecond timestamps.
Scale-set creation uses GitHub's label acceptance and disables runner
updates. The image release process controls runner versions.
The App service reconciles label creation and retirement together. Label
create and delete, maintenance, resume, and reinstall use that service.
Each reconcile upserts current remote IDs while preserving the
internal UUID, advisory lock key, and retained job/runner references.
A saved live set that reports a different group returns
ScaleSetGroupChanged. The adapter makes no unproven remote move and deletes
no unknown or retained resource. That case needs live move proof before
automatic repair can be enabled. It does not erase retained job history.

Before preparing active bindings, reconciliation releases the names of
deleted labels by their exact remote IDs. It renames a binding while tracked
work, pending delivery, or a listener lease remains. Otherwise it owns that
listener lease, checks tracked work again, removes the remote resource, and
records cleanup. It retains the binding and its job history. A missing old
remote resource counts as released. Authentication and provider failures
stay errors.

New label validation uses this same reconciler before the API saves the
label. Later reconciliation saves the prepared binding with activation.
Scheduler snapshots read those saved bindings. They use no unsaved remote
preparation result.

`RegistrationService::create` captures the context used by its remote
request. Its `RegisteredRunner` response returns that exact target with
the JIT configuration and runner ID. Runtime callers save the returned
target before delivering the configuration. A later binding read cannot
reconstruct the registration target.

[Binding reconciliation](src/app/bindings.rs) owns creation and retirement.
[Registration storage](../ciproxy-store-interface/registration-targets.md)
describes capture and older records.

## Workspace ownership

All workspaces of one GitHub organization share its managed runner group,
so the scale sets of two workspaces can claim the same label name. A scale
set therefore serves one workspace at most. The selected workspace is the
workspace that a preparation serves.

Before preparation, reconcile reads the current and retained registrations
of the GitHub organization in every workspace. It renames each saved scale
set of another workspace to a stable retired name and label. Unlike the
release of a deleted label, this path never removes such a set. Its running
jobs and retained idle runners stay, and they take no new job. A failed
rename fails that preparation.

Reconcile restores the name and the labels of each saved set of the
selected workspace to its label name. A saved remote ID that another
workspace also claims is foreign. Reconcile gives that shared set a durable
quarantine name and marker label on GitHub. Later retirement renames keep
the quarantine state. A saved-ID ensure never adopts a marked set. It
creates or recovers a fresh current set for that label, so a later return
cannot restore the shared set. A started job on the old remote ID gets its
real completion from
[job completion recovery](run-metadata.md#job-completion-recovery).
GitHub keeps the marker, so the store keeps no extra record. This path
removes no runner.

Before each remote mutation, reconcile chooses the final retirement name
and marker label. One selector checks the provider's existing resources
and every requested customer label. It keeps the chosen name and marker
off the requested labels, also when a collision needs a suffix. A label
that uses an internal prefix remains valid.
[App removal](../../docs/protocol/app-removal.md#removing-the-app) gives
the workspace rule.

[free-groups]: https://github.blog/changelog/2024-10-17-actions-runner-groups-now-available-for-organizations-on-free-plan/
[group-api]: https://docs.github.com/en/rest/actions/self-hosted-runner-groups#create-a-self-hosted-runner-group-for-an-organization
[rest-openapi]: https://github.com/github/rest-api-description
[arc-scale-sets]: https://docs.github.com/en/actions/how-tos/manage-runners/use-actions-runner-controller/deploy-runner-scale-sets#high-availability-and-automatic-failover
[runner-group-labels]: https://docs.github.com/en/actions/how-tos/write-workflows/choose-where-workflows-run/choose-the-runner-for-a-job#example-combining-groups-and-labels
