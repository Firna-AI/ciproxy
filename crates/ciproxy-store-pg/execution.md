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
Guarded result writes lock their guard rows and compare them again in one
transaction. Any change rejects the result and saves nothing. A late
registration can still save its actual GitHub runner ID and remote
scale-set ID for cleanup. It permits the start only while the guard matches
and the organization is admitted and active.
