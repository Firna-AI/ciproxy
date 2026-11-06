# Runner routing and retirement

This guide gives how the hosted runtime routes jobs to own-machine
runners, keeps their reservations, and retires them. The
[crate README](README.md) gives the rest of the control plane.

## Routing and reservations

Reconciliation runs every ten seconds.
Routing reads one repeatable-read workspace snapshot.
A dedicated PostgreSQL session lease serializes execution decisions.
The snapshot gives the pure scaler machine capacity, durable demand,
current labels, and the last 24 hours of default-label job history.
Warm shares follow proportional counts and the documented leftover order.
Docker spare uses a conservative upper bound on feasible root allocations.
Some mixed-label machines can receive fewer sticky disks.
The bound preserves root starts without an exact knapsack search.
Every accepted VM keeps an exact set of logical slot indices.
The same indices cross the launch command, agent state, and reports.
Non-contiguous free slots remain usable.

The original free-size request and actual total stay separate in reports.
The shared `RootSizing` helper applies the filesystem allowance once.
An incorrect release length can reduce the free disk promised to a job.

A runner reservation keeps its organization, machine, label request,
byte-precise allocation, GitHub runner identity, and delivery command.
The database never stores its plaintext JIT configuration or runtime token.
An interrupted JIT response can be recovered by its unique runner name.
The runtime retires that registration before it releases the reservation.
A busy removal result preserves the job and its VM.
A successful stop command still holds capacity until the VM is absent.
An offline removed machine still loses idle GitHub registrations.
One failed action does not block independent organizations.

Ordinary retirement needs positive proof from GitHub.
`RunnerRegistrations::retire` returns `Removed` after GitHub deletes the
exact runner or reports it gone, `Absent` when a live lookup finds no
runner of that name, or `Busy` with the runner ID.
Only `Removed` or `Absent` permits an ordinary stop. `Busy` saves the
trusted ID and the `Busy` state, with no stop.
Token, authentication, transport, and provider errors stay errors.
The local App state grants no result, so unavailable App access keeps an
idle runner reserved.
The store saves the trusted GitHub ID and `Stopping` in the same
transaction as the stop command.

Each machine keeps its last full report with the time that report arrived.
Register, Hello, and Report set that time. Agent events change only the
contact time of the machine.
A runner gets its first confirmation from positive evidence of its VM.
A current command result saves it with that result. Other evidence comes
from a full report that lists the runner, from the agent's ready, job start,
or exit event, or from GitHub.
The GitHub evidence is an eligible trusted job start or a busy answer for
the runner. Ignored stale `Started` facts grant no new evidence.
Each such observation saves the runner state, the trusted GitHub ID, and the
first confirmation in one transaction. The store writes that time once.
A full report confirms at its receipt time. Agent and GitHub observations
confirm at the time that the runtime handles them. The GitHub registration
saved before delivery confirms nothing.
A full report that omits a confirmed runner releases it only when the report
arrived after that confirmation. Absence alone never stops an unconfirmed
start.

Retirement saves `Stopping` and its stop command before delivery.
Each pass resends every saved stop with its command ID, after report
reconciliation and before job routing and new actions. Removed machines
get no resend.
A resend makes no GitHub call, and one failed send does not block another.
An applied result ends delivery. The capacity stays held until a later full
report omits the VM, or the runner exits.
`RunnerReady` and stale GitHub `Started` facts cannot replace `Stopping`.
A guest `JobStarted` event or a running VM in a report keeps the job `Busy`.

An agent report can mark a `Starting` runner with `configuration_lost`.
The agent sets it only when the guest confirms no configuration and the
agent holds no pending JIT. The runtime then retires that runner GitHub-first.
A busy result preserves it, and its resources stay held until the VM is absent.
An unconfirmed start is also retired GitHub-first when, more than 60 seconds
after its issue, a full report received after that issue omits it.

The guest supplies only a runner and job identity to the hosted process.
The runtime matches that identity to durable GitHub demand.
The GitHub API supplies repository, event, default branch, and head data.
The original runtime token stays on the customer machine.
The machine cache checks its signed scope outside the VM.

## Removal and take-out

The user selected immediate idle stop for machine removal and App removal.
A job assigned before the trusted guest handoff can fail in that window.
The runtime preserves jobs the guest has already observed.
Ordinary retirement and Drain still require GitHub removal first.
An owner's App removal saves a stop for every live reservation of that
installation, also a busy or starting one, with no GitHub call.
The event counts as accepted only after those stops are saved.
A lease conflict or a failed save leaves the event pending.
After a reinstall, older reservations use GitHub-first retirement.
Reservations issued after the removal stay untouched.
A workspace take-out keeps ordinary job protection.

A take-out callback holds the operation lock and the workspace lease.
A lease conflict returns an error, so the event stays pending.
The callback checks the organization, the GitHub account, the
installation, and its state. A stale event runs ordinary reconciliation
after the lease ends. It grants no authority over a newer installation.
A matching take-out first looks up its exact installation with the App
JWT. A successful lookup keeps ordinary GitHub-first retirement.
A 404 for that installation while the take-out is pending proves that the
owner removed the App, because ciproxy has not uninstalled it yet. The
callback then applies the owner-removal policy, with no registration call.
A 403, a token-route 404, or another lookup error grants no authority.
Only the reservations of that organization enter the callback.
Busy jobs, saved stops, and stopped reservations stay protected.
Each other live reservation needs positive retirement or a busy answer.
One failed runner does not stop the others, but an unanswered reservation
makes the callback return an error.
A delivery failure after a saved stop does not reject the event. Recovery
resends that stop with its command ID and issue time, with no GitHub call.
A completed take-out marker never permits a stop. An unknown older idle
state keeps its capacity.
