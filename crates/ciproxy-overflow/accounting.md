# Overflow accounting

Spend includes all ciproxy sandboxes across the workspace's organizations
and retained key versions.
It excludes other use of the Boat account.
The service saves the highest published start fee before dispatch.
The existing admission read selects the maximum
`overagePrices.startBrackets[].dollars` from `GET /limits`.
Missing or empty brackets fall back to `nextStartDollars`.
`start_fee_bound_usd` stores this one selected amount.
Later account reads do not replace that saved amount.
There is no fee reconciliation.
It reads each sandbox's runtime for explicit UTC calendar-month windows.
It uses the pinned hourly price for the known running interval between the
last observation and a confirmed normal deletion.
It counts a start fee only in its start month.
Measured dollars and the pinned rate define the continuous billing interval.
Its end is separate from the later lifecycle reconciliation time.
An impossible interval clamps to its observation bounds and logs a warning.
Every overflow-to-metering write uses a finite measured interval.
Its constructor clamps the end to the latest provider observation.
Later metering refreshes cannot add runtime beyond that prefix.
`Spend` separates confirmed costs, pending bounds, and expired settled bounds.
The scaler and start admission count all three against the monthly limit.
Each bound uses the requested size's pinned hourly price for the complete
six-hour lifetime, or two hours on a trial, plus the saved start-fee bound.
Measured runtime reduces the remaining runtime bound without counting twice.
A final runtime reading removes the remaining runtime bound.
The saved fee stays counted in its UTC start month after runtime completion
and lifetime expiry, including a free start with zero measured runtime.
Boat reports no actual start fee.
The saved bound never becomes a provider-confirmed start charge.
Savings count the confirmed start fee when present, or the saved fee bound.
Runtime still needs measured cost facts.

A lifetime expires for budget purposes from the saved dispatch time until a
trusted creation reading supplies the real start time.
Later running evidence prevents stale budget settlement, including when a
monthly cost read fails. A total runtime reading survives that failure.
A total within one UTC month supplies exact confirmed spend for that month.
An unallocated total across months supplies a conservative budget bound,
after subtraction of measured costs from other months.
That bound supplies no monthly measured charge or metering interval.
Each bound splits runtime at UTC month boundaries
and counts the fee only in its start month.
At budget expiry, the remaining bound moves to settled spend.
That transition changes no provider identity, lifecycle end, or key custody.
An uncertain create still requires recovery and retains its old key.
Later running evidence that contradicts a known lifetime prevents cleanup
from treating that old deadline as proof that the sandbox ended.
A later real cost replaces the estimate, including costs above its bound.

New starts continue while the limit covers existing costs and the proposed
start's bound in each UTC month its lifetime touches.
Dispatch locks the current saved settings and compares those exact monthly
amounts before it fixes the provider parameters.
Lowering the limit during provider or encryption work therefore blocks that
dispatch when the new limit cannot cover it.
The pending demand retains its identity and sealed input while it waits.
A missing safe fee bound returns `Error::AccountingIncomplete`.
`Spend::unbounded_starts` identifies the affected deliveries.
A new start with neither fee source waits before provider dispatch.
At the monthly limit, new starts wait until there is room.
Running jobs and deletion continue.

Missing or failed usage reads invalidate stale runtime completeness.
Known costs and saved account metadata remain readable after key rejection,
including records that still reference an old key.
Ordinary measured completion permits another start under the limit.
The store also protects metering from incomplete price intervals.
It publishes a bounded known billing prefix when a final interval is unknown.
The metering refresh can then account known costs and purge expired totals.
Relevant savings reads still return a typed incomplete error.
Ranges after a confirmed lifecycle end do not include that missing interval.

[Overflow service guide](README.md) gives configuration and integration.
[Spend limits](../../docs/protocol/overflow.md#spend-limit) gives the protocol.
