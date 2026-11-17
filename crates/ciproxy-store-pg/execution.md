# Machine and execution queries

This guide gives the gateway queries and the own-machine execution queries
of the PostgreSQL store. The [crate README](README.md) gives the rest of the
adapter.

## Gateway queries

Gateway queries lock the workspace before registration or token issuance.
They serialize against last-App removal and reject a retained empty workspace.
Registration inserts the machine, report, and hashed credential together.
It holds the install-token row until that transaction commits.
Independent token revocation waits for that registration before it returns.
Registrations that validate the token after revocation cannot use it.
The full last report uses JSONB, a connection identifier, and the receipt
time that only a full report writes. Snapshots read both from one row.
Replaced sessions and removed machines cannot overwrite it.
Desired drain and warm settings stay separate from observed report fields.
Machine reads keep machine fields and disks in one committed view.
Gateway snapshots include the full report from that same view.
A shared read snapshot prevents mixed report generations.
Contact writes contain only observed columns.

## Own-machine execution

Own-machine queries save runner and command reservations atomically.
A repeatable-read routing snapshot keeps reports, demand, and settings
consistent. It also reads the Boat account, deliveries, registration
identities, and the runner rows of Boat sandboxes in the same transaction.
A separate negative advisory-key range owns execution leases.
Mutable machine, organization, and label authority is checked again under
row locks before a new reservation commits.
The command records retain exact byte dimensions and logical slot sets.
No conversion through integer GB changes the held root allocation.
The first confirmation of a runner is written once, and retirement keeps it.
An observation saves the runner state, its GitHub ID, and that first time in
one transaction. A missing runner or command record rolls back both writes.
An applied command result stays a separate value.
A retirement saves the trusted GitHub ID, `Stopping`, and its stop command
in one transaction.
Worker job binding locks the exact registered machine runner before it
updates the command. The first write saves the opaque ID. An identical
replay preserves it. A different ID, invalid scope, missing command, or
stopped runner returns false. State and command updates preserve the saved ID.
Guarded result writes lock their guard rows and compare them again in one
transaction. An own-machine result needs the complete saved guard. A Boat
result needs only its lifecycle authority: the organization, the
registration intent, the runner, its jobs, and a `BoatDeliveryGuard`. That
delivery part holds the fixed inputs, phase, sandbox ID, end, and end
report of the delivery. A usage or billing reading, a measured start, or a
lifetime bound does not reject a Boat result.
`apply_boat_runner_result` returns `BoatRunnerWrite::Applied` with the
exact guard after the write, `Retired` after it removes the registration
intent, or `Rejected`. A rejected result saves nothing, with one
exception. A late registration can still save its actual GitHub runner ID
and remote scale-set ID for cleanup. It permits the start only while the
guard matches and the organization is admitted and active.
