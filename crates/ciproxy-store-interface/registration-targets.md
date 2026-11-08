# Registration targets

`scale_sets::RegistrationTarget` identifies one organization, shared label,
stored binding, and remote GitHub scale set. A binding can keep its row ID
when its remote resource changes. The remote ID therefore stays part of a
runner's registration target.

`RunnerStore::save_runner_registration` saves the target returned by the
GitHub registration service. It saves the GitHub runner ID in the same
write. `runner_registration_target` reads that captured value. It returns
`None` for an older runner whose remote target was never recorded.

`OwnRunner.registration_scale_set_id` exposes the captured remote ID beside
the runner's existing organization, label, and binding IDs.
`BoatRunnerIntent.registration_scale_set_id` exposes the same field for
Boat startup and recovery. Its JSON reader accepts an absent field as
unknown. The Boat registration write saves the response in the intent and
the materialized runner together.

Lifecycle observations update runner IDs and states without changing the
captured target. A Busy response gives no registration-time remote ID.
It cannot fill an older unknown target. Routing never reconstructs a
target from the binding that happens to be current later.

The coherent routing snapshot includes active saved bindings. The runtime
uses those bindings for labels and waiting jobs. It uses each runner's
captured target for supply. Unknown and obsolete idle supply retires
through ordinary cleanup. Running jobs stay protected.

## Migration

`202611081000_registration_targets` adds nullable
`runners.registration_scale_set_id`. Existing rows stay null. The migration
does not read current bindings or backfill a guessed target. Existing
Boat JSON remains unchanged and reads as unknown.

Apply the migration before the new runtime writes registrations. The
PostgreSQL store factory applies it at connection time. Replace all runtime
writers together so older writers cannot create more uncaptured runners.

Rollback removes the new column and its captured values. Roll back the
runtime first. The older runtime ignores the additive Boat JSON field.
Keep a database backup when the captured history must survive rollback.

[Runtime validation](../ciproxy-control-plane/binding-validation.md) gives
the operator commands. No JIT configuration enters these records.
