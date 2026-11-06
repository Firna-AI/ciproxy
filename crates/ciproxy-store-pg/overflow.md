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
