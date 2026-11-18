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

## Job bindings and evidence

`OwnRunner.confirmed_at` keeps the time of the first execution evidence of a
runner, from a command result or a trusted agent or GitHub observation.
`observe_own_runner` takes a required `OwnRunnerObservation`: the state, the
GitHub ID, and the observation time. In one transaction, it saves the state
and the ID, and it sets `confirmed_at` only when it is empty.
It filters by workspace, organization, and runner only. Without the runner
or its command record, it returns `NotFound` and saves nothing.
`record_own_command_result` saves the result of the current command. It
also saves that first evidence, and only an applied result ends delivery.
`retire_own_runner` saves the trusted GitHub ID, `Stopping`, and the stop
command in one transaction.
`bind_own_job` saves one immutable Worker job ID for the exact registered
machine runner. It accepts identical replay and returns false for conflicts
or a stopped runner. `OwnRunner.worker_job_id` retains that ID after shutdown.

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
