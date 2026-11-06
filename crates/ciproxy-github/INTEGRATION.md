# GitHub service integration

These boundaries extend the [crate guide](README.md).

## Service integration boundaries

The crate consumes the hosted store through `dyn Store` and blocking
workers. The later control-plane runtime supplies these adapters:

- `InstallAuthority` resolves exact installation claims and grants the first
  Admin role only for a new workspace. The sign-in service owns memberships
  and role storage.
- `LifecycleOutput` stops idle organization runners and their listeners.
  It preserves running jobs during removal and suspension.
  Each `Removed` event carries a required `RemovalCause`. `Owner` and
  `WorkspaceTakeout` select different retirement policies.
- After the last organization leaves, the store revokes machine and
  install credentials. The gateway closes each machine connection at its
  next contact. The output requests idle sandbox cleanup and
  schedules the subscription end. The overflow service keeps retired Boat
  keys until running sandbox cleanup finishes.
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

The [run metadata guide](run-metadata.md) gives trusted run facts,
assignment preparation, and webhook delivery recovery.
