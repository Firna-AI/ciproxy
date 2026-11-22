# Boat overflow in the hosted runtime

This guide gives how the hosted runtime runs jobs on Boat when the own
machines are busy. It gives the configuration, the runtime guarantees, and
the sandbox exit callback, and it links the live smoke checks. The
[crate README](README.md) gives the rest of the control plane.
[ciproxy-overflow](../ciproxy-overflow/README.md) owns sandbox delivery,
encrypted keys, costs, and cleanup. The
[overflow protocol](../../docs/protocol/overflow.md) gives the requirements.

## Configuration

`BOAT_SHARED_SNAPSHOT` is optional. It names the shared Boat snapshot or the
share code of the job image. The
[image guide](../../image/README.md#boat-shared-snapshot) creates the
snapshot and writes its share code to a private file. Keep that code out of
tracked files.

Use the snapshot of the same image release as `JOB_IMAGE_MANIFEST_PATH`.
The root bytes of that manifest size each new Boat root. The process does
not compare the two settings. Change them together.

An absent or empty value turns the Boat service off. Own machines still run
jobs. The dashboard API then reports `available: false`. Boat connection
changes then return HTTP 503, and the exit callback route is absent.

With a value, startup also builds the Cloud KMS client. `BOAT_KMS_KEY` must
then have the form `projects/P/locations/L/keyRings/R/cryptoKeys/K`.
Otherwise startup stops. The process gets KMS access tokens from the GKE
metadata server, so a key connection needs that server.

The runtime derives the sandbox mark and the callback URL from `PUBLIC_URL`.
It removes a trailing slash first. Boat sandboxes must reach
`PUBLIC_URL/hooks/boat/runner-exit` over HTTPS.

Keep the value set while a Boat sandbox can exist. Without it, the runtime
does no sandbox cleanup and serves no exit callback. Boat then ends each
sandbox only at its lifetime. That is six hours, or two hours on a free
trial. The deployment reads the optional value from its Kubernetes Secret.
[Boat setup](../../deploy/runtime.md#boat-shared-snapshot) gives the steps.
[Runtime setup](../../deploy/runtime.md#turn-on-the-runtime) gives the other
settings.

## Runtime guarantees

The routing snapshot of a workspace also holds the saved Boat account,
deliveries, registration identities, and runner states. The common snapshot
builder turns these records into Boat supply, if a Boat service is
configured. So the pure scaler sees all accepted Boat supply, also while
Boat is unavailable. It starts no second runner for that demand before the
[ready deadline](../../docs/protocol/overflow-sandbox-end.md#the-ready-deadline)
of the first runner.
The scaler applies own machines first, the
[overflow wait](../../docs/protocol/job-routing.md#the-overflow-wait), and
the Boat size rules. The runtime executes each Boat start through the
overflow service. Each Boat request runs in its own task, as the
[remote request rules](runners.md#remote-requests) say. An ordinary pass
starts no new Boat runner. After each successful Boat refresh, one more
pass applies only Boat actions, with no new refresh.

- The runtime saves a registration identity before it asks GitHub for a JIT
  registration. The identity holds the runner, delivery, organization,
  installation, label, and scale-set IDs, the sizes, and the required root
  bytes.
- The JIT configuration goes only into the encrypted delivery input. The
  database never holds it in plaintext.
- An unfinished delivery resumes with the same IDs after a restart or an
  uncertain result.
- A registration without a live delivery is retired GitHub-first by its
  unique runner name, `ciproxy-<runner ID>`. The runtime then deletes its
  identity.
- When the GitHub access of an identity has ended, recovery defers with no
  GitHub call and keeps the identity. Access ends at saved owner removal,
  at a completed take-out of the current installation, or when the
  organization is absent. A new install of a retained organization ends
  the deferral, and ordinary retirement resumes. A completed take-out of an
  older installation grants nothing over the new installation. A later
  completed take-out of the new installation also applies to the older
  identities.
- Identity cleanup deletes a terminal identity when its saved ownership
  allows it. An identity is terminal only when no unfinished delivery and
  no live runner use it. A completed take-out allows deletion at once, but
  it never permits a sandbox stop. Owner removal allows deletion at the
  30-day deadline of the organization. The organization purge keeps that
  deadline on each identity with a saved installation ID that is not
  terminal yet. Identity cleanup
  keeps an identity without a saved installation ID. It runs each pass with
  no GitHub call, also while the Boat service is off. It deletes no
  delivery, runner, or job history. The
  [store guide](../ciproxy-store-pg/overflow.md#registration-ownership)
  gives the exact rule.
- An accepted delivery keeps its original request and its required root
  bytes, also after a new image release.
- A dispatched delivery keeps its fixed provider size. An unsent delivery
  can still select the current catalog size.
- A trusted GitHub job start marks the Boat runner busy. A completed job
  marks it exited, so it cannot take a second job while deletion waits.
  Without such a job fact, a saved `Busy` state also marks the runner busy.
  Each runner takes one job, so a busy runner covers no new waiting job.
- Idle retirement keeps a runner that a running job or waiting demand needs.
  Waiting demand keeps a launched runner only until its ready deadline. The
  scaler can then start another runner for that demand by the usual
  placement. Each retirement removes the GitHub registration first. A busy
  answer keeps the sandbox.
- Waiting demand does not keep a saved `Busy` runner without a started job.
  Each cleanup pass asks GitHub to remove it. A busy answer keeps it.
- Take-out retires idle Boat supply before the API removes GitHub access.
  It waits for saved retirement or a current busy answer for every saved
  reservation. A saved provider deletion can finish after acceptance.
- After an owner removes the App, launched sandboxes stay until their
  runner exits or their saved lifetime ends. Missing job facts do not prove
  that a sandbox is idle. The runtime checks for ended GitHub access before
  and after GitHub retirement. A successful GitHub response does not permit
  early sandbox deletion after removal.
- App removal stops new Boat starts. The last organization removal retires
  the Boat key and holds it until the unfinished sandboxes end. Exit
  reports still delete their sandbox, as [Exit callback](#exit-callback)
  says. App removal is rare, and the lifetime and spend limit bound its
  cost, as
  [Removing the App](../../docs/protocol/app-removal.md#removing-the-app)
  explains.
- A new installation clears saved owner removal. Old idle sandboxes can
  then retire through their retained key before Boat connects again.
  Running jobs keep their old delivery and key after a new connection.
- Old idle supply cannot cover a replacement label with the same name.
  Retired labels and removed bindings cannot start an undispatched delivery.
- When Boat or its cost data fails, accepted Boat supply stays reserved and
  new Boat starts wait. Own-machine routing continues.

## Exit callback

`POST /hooks/boat/runner-exit` takes the runner-exit report of a sandbox.
The route uses no browser session. The guest sends
`GuestToAgent::RunnerExited` as JSON with `Authorization: Bearer TOKEN`.
Each sandbox has its own token, which works only for its own runner.

The route first saves the report as a one-use claim. The claim needs no
workspace lock, so other Boat work of the workspace cannot drop it. The
route then waits up to 5 seconds for that lock and deletes the sandbox in
the request. When the wait ends first, or a step after the claim fails,
the route still answers 204, and a later cleanup pass deletes the sandbox.
Later delivery saves keep the claim and the first recorded end.
[The exit report](../../docs/protocol/overflow-sandbox-end.md#the-exit-report)
gives the requirements.

| Status | Meaning                                                                                               |
| ------ | ----------------------------------------------------------------------------------------------------- |
| 204    | The control plane saved the report. It deletes the sandbox in the request or at a later cleanup pass. |
| 400    | The message is not `RunnerExited`, or its runner ID is not a UUID.                                    |
| 401    | The token is missing, wrong, already used, or for another runner, or its delivery already ended.      |
| 503    | The request failed before it saved the report.                                                        |

[The ready deadline](../../docs/protocol/overflow-sandbox-end.md#the-ready-deadline)
and
[After an App removal](../../docs/protocol/overflow-sandbox-end.md#after-an-app-removal)
say what happens to the sandbox of a launched runner with no report and no
job. Boat ends each sandbox at its lifetime at the latest.

## Live smoke checks

The [live smoke guide](overflow-smoke.md) gives the live Boat checks of a
deployed control plane. It gives their setup, credential file, commands,
local tests, and cleanup.

### Local tests

The [live smoke guide](overflow-smoke.md#local-tests) gives the local tests
of the cleanup delete requests.
