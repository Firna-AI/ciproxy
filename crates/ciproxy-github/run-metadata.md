# Trusted run metadata

This guide gives the run facts that the App integration reads from
GitHub, the assignment facts for the agent, the cache entry metadata, and
webhook delivery recovery. The [crate README](README.md) gives the rest of
the integration.

## Run facts

Run facts come from the Actions read API for the message's repository and
run ID. Known runs reuse their durable facts under workspace and
organization scope. A repository removal, rename, or later API outage does
not prevent known completion persistence and acknowledgement. A new run
whose first facts read fails still leaves its batch unacknowledged.
Saved facts preserve `event`, `head_branch`, `head_sha`, and nullable
`head_repository` from that API. A reusable workflow path stays separate.

## Assignment facts

The API does not supply a guaranteed full job ref. Its `head_branch` does
not distinguish a same-name branch and tag. Associated pull requests do
not identify the triggering pull request. A deleted head repository stays
absent. This crate makes no event-to-ref conversion and grants no cache
or Docker trust from these fields.
`JobAssignments::facts` returns a `JobPreparationOutcome` for each accepted
assignment. `TrustedFacts` prepares the expected runtime-token identity for
the agent. It combines the scale-message run ID and job UUID, the JIT
runner ID, the API repository identity, event, and head repository, and
the saved repository default branch. `AssignmentService` consumes the
existing tenant-scoped repository snapshot through `AssignmentStore`.
When that snapshot loads but lacks the repository, the result is
`GitHubOnly` with the snapshot's organization ID. The job then uses GitHub
directly. A snapshot or store error stays an error, so the caller retries.
It never guesses a default branch or job ref.

The agent sends those facts and the original token to the machine cache.
The machine returns a checked signed scope or a typed unknown reason.
It checks cache trust, also for reusable workflows. The agent uses the
checked ref for Docker policy and owns the Docker-disk lifecycle.
Installation tokens keep their three permissions for API event and head
metadata. The App's fourth permission, "Members: read", serves sign-in only.

## Cache entry metadata

`CacheEntries::cache_entries` reads the cache entry list of one repository
with its installation token. The existing "Actions: read" permission
covers this list. The adapter reads every page of the list on each call,
and it keeps no copy between calls. Each `CacheEntryMetadata` keeps the GitHub entry ID, the full ref, the
key, the version, and the creation time. Refs stay unchanged, also tag
refs. The creation time becomes decimal-string Unix nanoseconds. A failed
page or an unreadable time fails the whole call, so no partial list leaves
the adapter. It reads no cache contents.

The control plane calls this adapter only for the repository of a job that
runs on the calling machine. The
[cache protocol](../../docs/protocol/cache.md#restore-order) gives that
rule and the use of these facts.

## Webhook delivery recovery

The hosted runtime can recover failed webhook deliveries through the
App-JWT delivery-history API. Recovery reads every cursor page and groups
attempts by event GUID. A successful latest retry prevents an older failure
from being redelivered again. Individual redelivery failures remain
retryable without blocking the other retained events.
This uses no extra installation permission.
