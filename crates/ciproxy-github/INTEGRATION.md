# GitHub service integration

These boundaries extend the [crate guide](README.md).

## Service integration boundaries

The crate consumes the hosted store through `dyn Store` and blocking
workers. The later control-plane runtime supplies these adapters:

- `InstallAuthority` resolves installation intents and grants the first
  Admin role. The sign-in service owns memberships and role storage.
- `LifecycleOutput` stops idle organization runners and their listeners.
  It preserves running jobs during removal and suspension.
- After the last organization leaves, the store revokes machine and
  install credentials. The output closes machine connections, requests
  idle sandbox cleanup, and schedules the subscription end. The overflow
  service keeps retired Boat keys until running sandbox cleanup finishes.
- `JobOutput` notifies the snapshot and scaler service of saved demand.
  That service maps GitHub runner IDs to its runner records.
  It supplies consistent snapshots and sends accepted actions to agents.
- The retention worker calls `purge_github_jobs` for completed demand after
  90 days, including records with pending notifications. Active work stays.
  When GitHub omits `finishTime`, expiry uses the first trusted completion
  observation from the store adapter's injected `Runtime`. Replay keeps
  that observation. The GitHub completion field stays absent.

The control-plane runtime calls `pending_lifecycle_organizations` after
restart and while retrying failures. It calls `replay_lifecycle(github_id)`
for each result independently. Replay needs no GitHub read or surviving
registration. It takes the same organization mutation lease as webhooks.
The runtime must keep retrying failed deliveries. This crate exposes the
entry points and does not start a background worker itself.

No adapter in this crate calls Boat.
An owner removal cannot delete inaccessible GitHub resources.
Suspension preserves the workspace and its credentials.
Unsuspend reconciles the repositories and resumes lifecycle delivery.
The dashboard take-out service owns the intentional-removal distinction.

## Trusted run metadata

Run facts come from the Actions read API for the message's repository and
run ID. Known runs reuse their durable facts under workspace and
organization scope. A repository removal, rename, or later API outage does
not prevent known completion persistence and acknowledgement. A new run
whose first facts read fails still leaves its batch unacknowledged.
Saved facts preserve `event`, `head_branch`, `head_sha`, and nullable
`head_repository` from that API. A reusable workflow path stays separate.

The API does not supply a guaranteed full job ref. Its `head_branch` does
not distinguish a same-name branch and tag. Associated pull requests do
not identify the triggering pull request. A deleted head repository stays
absent. This crate makes no event-to-ref conversion and grants no cache
or Docker trust from these fields.
`JobAssignments::facts` prepares the expected runtime-token identity for
the agent. It combines the scale-message run ID and job UUID, the JIT
runner ID, the API repository identity, event, and head repository, and
the saved repository default branch. `AssignmentService` consumes the
existing tenant-scoped repository snapshot through `AssignmentStore`.
It returns no assignment when that snapshot lacks the repository. The job
can still use GitHub directly. It never guesses a default branch or job ref.

The agent sends those facts and the original token to the machine cache.
The machine returns a checked signed scope or a typed unknown reason.
It checks cache trust, also for reusable workflows. The agent uses the
checked ref for Docker policy and owns the Docker-disk lifecycle.
The App keeps all three permissions for API event and head metadata.
