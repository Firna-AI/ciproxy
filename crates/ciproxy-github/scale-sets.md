# Repository access and scale sets

This guide describes how the GitHub App integration keeps repository
access, runner groups, and scale sets. The [crate README](README.md) gives
the rest of the integration.

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

Managed group names use the stable GitHub organization ID. Reconcile first
reads a saved managed group ID, including retained registrations, so an
owner rename does not create another group. It restores the managed name
and repository policy on that same group. After local data is purged, the
stable name finds the retained GitHub group again.
GitHub supports self-hosted runner groups on its Free plan, as its
[Free runner-group announcement][free-groups] confirms.
The live selected-group probe verified the selected-repository policy on
GitHub Free. GitHub runs the job of a selected repository on a runner of
the group. It keeps the job of another private repository queued with no
runner.

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

Before preparing active bindings, reconciliation releases retired names by
their exact remote IDs. It renames a binding while tracked work, pending
delivery, or a listener lease remains. Otherwise it owns that listener
lease, checks tracked work again, removes the remote resource, and records
cleanup. It retains the binding and its job history. A missing old remote
resource counts as released. Authentication and provider failures stay errors.

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

[free-groups]: https://github.blog/changelog/2024-10-17-actions-runner-groups-now-available-for-organizations-on-free-plan/
