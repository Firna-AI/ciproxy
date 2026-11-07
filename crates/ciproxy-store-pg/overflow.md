# Overflow store

This guide gives the PostgreSQL queries for Boat execution, its delivery
authority, and its accounting reads. The [crate README](README.md) gives
the rest of the store.

## Sessions and deliveries

Overflow uses one dedicated advisory-lock connection per workspace.
Its negative key range is separate from billing and scale-set locks.
A blocking callback lease waits on that dedicated connection.
Catalog observations and exact delivery records persist before Boat access.
Reservation rechecks the active key, workspace settings, organization,
label, and scale-set ownership under the workspace row lock.
Pending acceptance saves demand, sealed input, and the saved fee together.
It saves no fixed provider parameters or provider cost bound.
Pending connections receive fresh database-generated grant identities.
Activation consumes a matching live grant under the workspace row lock.
A later grant cannot restore an older operation, even with the same key ID.
Suspension and removal with another App present preserve shared grants.
One current startup query binds the saved delivery to its organization and
key in that workspace. It requires active App state, enabled overflow, and
the accepted non-retired key together.
The same authority SQL checks scale-set ownership, the label's Boat switch,
and exact original requested dimensions during reservation and later startup.
An undersized selected size also requires the label's current largest-size
fallback permission. Fitting requests remain permitted under Wait.
Each delivery retains its image root bytes and required total root bytes.
Authority compares catalog disk with those saved totals.
Even a largest-size fallback must hold the complete image.
The dispatch transaction accepts only the saved unsent delivery.
It rechecks authority and the current saved limit before saving `Creating`.
It compares the exact pending record, including its identity and saved fee.
It saves selected size, lifetime, and dispatch time in that same transaction.
The caller supplies those parameters with their required monthly amounts.
Current-label queries supply facts for service policy without selecting sizes.
Cancelled unsent records release sealed input without provider or cost facts.
Adoption saves the provider identity, immutable runner placement, and price
facts in one transaction.
Callback claims save token consumption and deletion intent together.
Refusal records and account restrictions also commit together.
Key erasure checks unfinished deliveries as well as live runners.
No query receives a plaintext key or runner registration.

## Runtime registration identities

The runtime saves each Boat registration identity before its GitHub JIT
request. The insert holds the workspace row lock. It saves nothing and
returns `Conflict` unless all of these facts hold:

- The organization has an active App and admitted execution.
- The workspace has overflow on and an accepted key.
- The current label allows Boat and matches the requested size.
- The scale set of that organization and label is active.

A row holds identities, sizes, the required root bytes, and the
installation ID of its organization at the insert. It holds no JIT
configuration or callback token. The saved GitHub runner ID also goes to
the runner row of the delivery, when that row exists. The routing snapshot
reads these rows with the account and delivery rows in one repeatable-read
transaction. The separate durable snapshot read takes no overflow session.
`save_boat_registration` saves the response's exact remote scale-set ID
with its runner ID. It updates the intent and an existing runner together.
Sandbox materialization copies this captured ID from the saved intent.
It never copies a later binding ID. Ordinary recovery and Busy writes
change the runner ID only. They preserve captured targets and leave
older unknown targets unknown.

### Registration ownership

The `boat_registration_ownership` view gives three facts for each row.
Registration recovery, identity cleanup, and the organization purge use
these facts.

- GitHub access has ended when the workspace has no row for the
  organization, or when the organization has saved owner removal. It has
  also ended when the organization has a removed App and a completed
  take-out of its current installation in the same workspace. An older
  take-out marker does not count after a reinstall.
- A row is terminal when each delivery record with its delivery ID or
  runner ID has ended and matches its workspace, delivery, runner, and
  organization. Each runner row with its runner ID must also match its
  workspace and organization and be stopped. A row without such records
  is also terminal.
- The cleanup time is the earliest time when cleanup can delete the row.
  Such a completed take-out makes it immediate. Owner removal makes it the
  30-day deadline of the organization. For an absent organization, it is
  the deadline that the purge copied to the row. Other rows, such as a
  legacy row without a saved installation ID, have no cleanup time.

The access read counts a missing row as ended, because no recovery work
remains. The cleanup query takes the workspace row lock. It deletes each
terminal row whose cleanup time has passed. It deletes no delivery, runner
row, or job history.

The organization purge holds the same lock. It copies the deadline of each
expired organization to the rows of that organization with a saved
installation ID. It then runs the cleanup query before it deletes the
organization. A row that is not terminal yet keeps the copied deadline. A
later cleanup deletes it after it becomes terminal.

## Accounting reads

Metering price facts require a trusted runtime observation and a counted
start fee. They select the confirmed fee, or else the saved fee bound.
That selection never confirms a provider charge.
Delivery JSON retains the admission bound and actual fee separately.
Their upserts correct provider start times and every price field.
Every overflow price row has a finite end from its measured interval.
Later metering refreshes cannot extend an old reading.
An unknown final interval publishes only the known measured billing prefix.
Boat accounting remains available so the metering refresh can reach purge.
Relevant savings reads return `AccountingIncomplete` while both fee sources
are missing or a required runtime interval remains unknown.
The savings range excludes a lifecycle end at its first instant.
Execution and cleanup do not use those reporting reads.
