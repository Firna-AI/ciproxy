# Trusted run metadata

This guide gives the run facts that the App integration reads from
GitHub, the assignment facts for the agent, the cache entry metadata, job
completion recovery, and webhook delivery recovery. The
[crate README](README.md) gives the rest of the integration.

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
assignment. Its caller supplies the `JobIdBinding` of the exact registered
runner: the runner's saved Worker job ID and the scale-set job UUID of the
saved demand. The
[runner guide](../ciproxy-control-plane/runners.md#job-assignment-and-worker-job-binding)
says how the control plane builds it. This crate copies the binding
unchanged, and no job name enters it. `TrustedFacts` prepares the expected
runtime-token identity for the agent. It combines the scale-message run ID,
that binding as `JobAssignment.job_ids`, the JIT runner ID, the API
repository identity, event, and head repository, and the saved repository
default branch. `AssignmentService` consumes the
existing tenant-scoped repository snapshot through `AssignmentStore`.
When that snapshot loads but lacks the repository, the result is
`GitHubOnly` with the snapshot's organization ID. The job then uses GitHub
directly. A snapshot or store error stays an error, so the caller retries.
It never guesses a default branch or job ref.

The agent sends those facts and the original token to the machine cache.
The machine accepts a signed `job_id` equal to either ID of the binding.
It returns a checked signed scope or a typed unknown reason.
It checks cache trust, also for reusable workflows. The agent uses the
checked ref for Docker policy and owns the Docker-disk lifecycle.
The REST adapter reads the API event and head metadata with its
installation token. [Credential scopes](README.md#credential-scopes) gives
the scope of each credential.

## Cache entry metadata

`CacheEntries::cache_entries` reads the cache entry list of one repository
with its installation token. The existing "Actions: read" permission
covers this list. The adapter reads every page of the list on each call,
and it keeps no copy between calls. Each `CacheEntryMetadata` keeps the
GitHub entry ID, the full ref, the key, the version, and the creation
time. Refs stay unchanged, also tag refs. The creation time becomes
decimal-string Unix nanoseconds. A failed page or an unreadable time fails
the whole call, so no partial list leaves the adapter. It reads no cache
contents.

The control plane calls this adapter only for the repository of a job that
runs on the calling machine. The
[cache protocol](../../docs/protocol/cache.md#restore-order) gives that
rule and the use of these facts.

## Job completion recovery

The REST timing repair reads the actual Actions job of a run and its
unique runner. It saves the real times and the numeric Actions job ID of
an attempt that still lacks a real start or finish. The control plane runs
it in its [accounting worker](../ciproxy-control-plane/github-workers.md),
with a backoff for each attempt.

A listener receives the messages of one remote scale set. When reconcile
moves a binding to a fresh remote set, its listener restarts on the new
remote ID. A started job on the old remote set then gets no completion
message. The repair therefore also selects each started attempt whose
exact runner registered on a previous remote ID of the binding. A started
job on the current remote ID stays out of this scan.

The runner can belong to a retained registration in another workspace of
the same GitHub organization. Recovery matches stable GitHub organization
and runner IDs. The attempt and its updates stay in its saved workspace.

For such an attempt, the repair polls GitHub with the same backoff until
GitHub reports the actual completion. It saves the terminal state, the
finish time, and GitHub's conclusion verbatim. The store's
[timing writes](../ciproxy-store-pg/github-jobs.md#actions-timing-writes)
say which saved values stay. This needs only the existing "Actions: read"
permission and no new service.

## Webhook delivery recovery

The hosted runtime can recover failed webhook deliveries through the
App-JWT delivery-history API. Recovery reads every cursor page and groups
attempts by event GUID. A latest attempt that got the route's
acknowledgement prevents an older failure from being redelivered again.
Every other latest attempt counts as failed. The
[GitHub worker guide](../ciproxy-control-plane/github-workers.md) gives the
acknowledgement status and its reason. Individual redelivery failures
remain retryable without blocking the other retained events.
Recovery uses only the App JWT, as
[Credential scopes](README.md#credential-scopes) says.
