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
The API also exposes scale-set deletion for the later label and take-out
services. Each reconcile upserts current remote IDs while preserving the
internal UUID, advisory lock key, and retained job/runner references.
A saved live set that reports a different group returns
ScaleSetGroupChanged. The adapter makes no unproven remote move and deletes
no unknown or retained resource. That case needs live move proof before
automatic repair can be enabled. It does not erase retained job history.

Before resume or retained reinstall prepares active label bindings, it releases
the names of retired bindings by their exact remote IDs. A missing old remote
resource counts as released. Authentication and provider failures stay errors.
This preserves label replacement without changing retained job references.

[free-groups]: https://github.blog/changelog/2024-10-17-actions-runner-groups-now-available-for-organizations-on-free-plan/
