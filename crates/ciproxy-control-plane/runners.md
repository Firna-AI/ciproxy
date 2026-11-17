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
It also gives the Boat account, its deliveries, the saved Boat
registrations, and the saved Boat runner states. The
[overflow guide](overflow.md) gives the Boat runtime.
The snapshot also reads active saved scale-set bindings. Labels, waiting
jobs, and start actions retain their exact registration target. Idle
supply counts only for the same organization, label, binding, and remote
scale-set ID. This applies to own machines and Boat.

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
The successful registration response supplies its exact remote target.
The runtime saves that response before it sends the JIT configuration.
It does not derive the target from the earlier routing snapshot or from
a later binding read. Reports, Busy results, and recovery preserve the
saved target. Older uncaptured targets stay unknown. Their idle runners
retire through ordinary GitHub-first cleanup. Running work stays protected.
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

The gateway callbacks put each agent event and each machine removal on one
queue, in arrival order, and return at once. The queue holds up to 1024
events. One supervised runtime task handles them one at a time. It skips an
agent event whose connection is no longer the current one of its machine.
When the handling of an event fails or panics, the runtime logs a warning
and handles the next event. The machine stays connected. The runtime logs a
warning when it drops an event because the queue is full. The next full
report and the saved-stop resend then repair the saved state.

When the agent sends `RunnerReady`, `JobStarted`, or `RunnerExited`, the
runtime saves that observation first.
Then it reads a fresh workspace snapshot, which holds the saved runner state.
It routes each saved GitHub job of the workspace with that snapshot.
A `RunnerReady` that cannot replace `Busy` or `Stopping` routes no job.
For `RunnerExited`, the runtime saves `Stopped`.
The runner stays `Stopped` while its saved GitHub job is still `Started`,
because job routing skips a `Stopped` runner.
The exit releases the slots and disk of that runner for the next pass, with
no new full report.

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

## Job assignment and Worker job binding

The guest supplies only a runner and job identity to the hosted process.
The GitHub API supplies repository, event, default branch, and head data.
The original runtime token stays on the customer machine.
The machine cache checks its signed scope outside the VM.

The runtime keeps three job IDs apart:

- The scale-set job ID is the `jobId` UUID of the scale-set messages.
  Saved GitHub demand uses it.
- The Worker job ID is the `JobId` UUID of the job request that the
  runner's Worker gets. The
  [trusted source hook](../../image/runner-patch/README.md) reads it before
  action setup, and the agent reports it.
- The Actions job ID is the numeric job ID of the GitHub REST API.

The Worker and scale-set job IDs can differ.

`Assignment::resolve` joins saved demand to its runner. The demand must be
`Started` or `Completed`, and name a GitHub runner ID. The runner must have
that GitHub runner ID, in the same organization and scale set. When two
runners match, it selects none. Own-machine job routing and the cache
metadata route share this join.

An agent `JobStarted` event and a full report that lists a running runner
carry the Worker job ID. The runtime binds that ID to the runner before it
routes jobs. The store saves the binding only for the exact runner record,
with the same workspace, organization, machine, and saved GitHub runner ID.
It refuses a `Stopped` runner. A JIT runner takes one job, so its first
saved ID never changes. An identical replay keeps it. A different ID
changes nothing, and the runtime logs a warning. Without a saved GitHub
runner ID, the runtime saves nothing and logs a warning. A later full report
retries. The saved ID stays after the runner stops and across restarts.

`Assignment::for_worker` authorizes the cache metadata route. It accepts a
job ID only when it equals the saved Worker job ID of a `Busy` or
`Stopping` runner on the calling machine. Exactly one `Started` demand must
join to that runner. The answer lists the cache entries of the repository
of that demand. The route never matches a request against a scale-set job
ID or a job name.

The binding does not change job preparation or `JobFacts`. Both carry the
scale-set job ID. The agent applies a preparation only to the runner that
reported the same job ID. It registers a cache session only when the
`JobFacts` job ID equals the Worker job ID. So a job whose Worker and
scale-set job IDs differ gets no preparation and no machine-cache session.
The [cache sidecar guide](../ciproxy-cache-sidecar/README.md#runtime-token-job-identity)
gives the source limit of the runtime token's `job_id` claim.

## Remote requests

The runtime changes reservations and runner states in short serialized
steps. The workspace lease serializes these steps across processes. The
runtime never holds that lease while it waits for GitHub or Boat.
Each GitHub or Boat request runs in its own supervised task. The runtime
logs each request that fails. Shutdown stops and joins these tasks and the
event queue. The runtime also cancels them when its supervisor ends with an
error or is cancelled.
Tick, job, agent, and lifecycle calls return after their saved-state steps.
They do not wait for the remote tasks that they start.
A slow request does not delay the reports of other machines, command
delivery, job updates, or other workspaces.
A reservation or a stop counts only after the store saves it. The
reservation saved before a request counts as supply, so a later pass starts
no second request for it.
Before an own-machine request starts, the runtime reads the current saved
guard of the reservation. It compares only identity and lifecycle with the
proposed work: the runner, organization, label, binding, GitHub ID,
captured registration target, placement, command ID, and state. Evidence
and measurements, such as the first confirmation, the acknowledgement, the
issue time, the slot set, and sizes, do not stop a dispatch.
An own-machine result saves only while the complete saved state that
allowed its request still matches, newer evidence included. A Boat result
saves only while the lifecycle authority of its records still matches.
So a late result cannot replace a newer report, command, runner state,
job, removal, or reinstall. It also cannot release capacity early.
The runtime sends a start or job preparation only while that state still
matches. A late GitHub registration keeps its actual identity for
GitHub-first cleanup by its unique name, and it starts no runner.

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

A take-out callback first checks the organization, the GitHub account, the
installation, and its state. A stale event runs ordinary reconciliation
instead. It grants no authority over a newer installation.
For a matching event, the callback takes the workspace lease for its
saved-state steps only. A lease conflict returns an error, so the event
stays pending.
A matching take-out then looks up its exact installation with the App
JWT. That lookup runs in a supervised task, outside the lease, and the
callback returns an error meanwhile. The task uses the answer only while
the saved organization still matches the event exactly.
A successful lookup keeps ordinary GitHub-first retirement for one pass.
Each pending pass that used it starts one fresh exact lookup, outside the
lease, with at most one lookup at a time for each organization. Only the
first lookup starts retirement. A fresh lookup changes only the saved
answer, and the next pass uses it. A confirming answer keeps the busy
answers of the current take-out check, and a failed check stays failed.
A 404 for that installation while the take-out is pending proves that the
owner removed the App, because ciproxy has not uninstalled it yet. That
answer stays final. The callback then applies the owner-removal policy,
with no registration call. An earlier failed Boat check does not block
that acceptance.
A 403, a token-route 404, or another lookup error grants no authority, and
the last answer stays.
Only the reservations of that organization enter the callback.
Busy jobs, saved stops, and stopped reservations stay protected.
Each other live reservation needs positive retirement or a busy answer.
Acceptance includes own-machine commands, Boat registration intents, Boat
deliveries, and materialized Boat runners. A Boat delivery that can still
launch keeps take-out pending. This includes an ambiguous dispatched start.
A retained running Boat delivery needs a current GitHub busy answer and a
saved `Busy` state. That answer must come from the supervised GitHub check
of the current take-out. Its lifecycle authority must not change after it.
That authority is the organization, the registration, the runner, its
jobs, and the fixed inputs, phase, sandbox ID, end, and end report of the
delivery. Routine usage and billing readings, measured starts, and lifetime
bounds do not change it. A saved `Busy` state or old job facts alone grant
no take-out answer.
A retired idle Boat delivery needs saved provider deletion and removal of
its registration intent. Registration-only reservations also need positive
GitHub retirement before their intent is removed.
The API removes scale sets and the App only after this callback accepts.
One failed runner does not stop the others, but an unanswered reservation
makes the callback return an error. A queued or running GitHub or Boat
request is not an answer. The lifecycle outbox then retries the same event
until each reservation has saved retirement or a current busy answer.
A delivery failure after a saved stop does not reject the event. Recovery
resends that stop with its command ID and issue time, with no GitHub call.
A provider deletion failure after Boat retirement also permits acceptance.
Cleanup retries its saved `Deleting` phase without another GitHub removal.
A completed take-out marker never permits a stop. An unknown older idle
state keeps its capacity.
After a completed take-out of the current installation, recovery makes no
GitHub call for the Boat registration records of that organization.
Identity cleanup can then delete such a record after its delivery and
runner end, as the [overflow guide](overflow.md#runtime-guarantees) says.

## Registration rollout and live validation

The [registration storage guide](../ciproxy-store-interface/registration-targets.md)
describes the nullable migration and rollback. Deploy all runtime writers
together, then restart their workers. Old idle runners can retire and
need replacement capacity. Their running jobs keep their original VM.

Run [the binding smoke](scripts/binding-smoke.py) with a private credential
file in a disposable test workspace. Its
[operator guide](binding-validation.md) gives the rollout observations,
replacement scenario, and resume and reinstall checks.
