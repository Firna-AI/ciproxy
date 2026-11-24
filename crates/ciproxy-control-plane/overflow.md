# Overflow in the hosted runtime

The hosted runtime runs jobs on the machines and providers of each label's
order. This guide covers provider configuration and the exit callback. The
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

An absent or empty value disables new Boat starts. Own machines still run
jobs. The dashboard API reports `available: false`. A Boat connection or
key change returns HTTP 503. Disconnect remains available. The runtime
serves the callback and maintains saved deliveries.

Startup builds the Cloud KMS client.
`OVERFLOW_KMS_KEY` must have the form
`projects/P/locations/L/keyRings/R/cryptoKeys/K`. The chart points it at
the existing `boat-account-keys` key. The process gets KMS access tokens
from the GKE metadata server.

The runtime derives the sandbox mark and the callback URL from `PUBLIC_URL`.
It removes trailing slashes first. New deliveries get
`PUBLIC_URL/hooks/overflow/runner-exit` as their HTTPS callback URL.

Boat ends each sandbox at its lifetime. That is six hours, or two hours on
a free trial. The deployment reads the optional value from its Kubernetes
Secret.
[Boat setup](../../deploy/runtime.md#boat-shared-snapshot) gives the steps.
[Runtime setup](../../deploy/runtime.md#turn-on-the-runtime) gives the other
settings.

## Runtime guarantees

The routing snapshot holds the workspace's provider accounts, deliveries,
registration identities, and runner states. The scaler sees saved supply
even while a provider refresh fails. The
[ready deadline](../../docs/protocol/overflow-sandbox-end.md#the-ready-deadline)
and the [scaler rules](../ciproxy-scaler/README.md#what-this-crate-does) say
when the scaler starts another runner for demand that this supply covers.
The scaler walks the saved order of each label and applies the
[overflow wait](../../docs/protocol/job-routing.md#the-overflow-wait) when
leaving own machines. The shared service uses the chosen provider's image
and factory. Provider requests run in supervised tasks, as the
[remote request rules](runners.md#remote-requests) say. New provider starts
and unsent retries wait for a current provider check. A job waits for the
check when a configured, connected provider precedes own machines in its
order. It can start or free warm capacity on an own machine after the
check if the provider cannot take it. Before starting a replacement, the
runtime withdraws the unsent delivery under the workspace session lock.
A delivery whose dispatch has begun still covers the waiting demand.

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
  no GitHub call, also while overflow is off. It deletes no
  delivery, runner, or job history. The
  [store guide](../ciproxy-store-pg/overflow.md#registration-ownership)
  gives the exact rule.
- An accepted delivery keeps its original request and its required root
  bytes, also after a new image release.
- A dispatched delivery keeps its fixed provider size. An unsent delivery
  can still select the current catalog size.
- A trusted GitHub job start marks the overflow runner busy. A completed job
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
- Take-out retires idle overflow supply before the API removes GitHub access.
  It waits for saved retirement or a current busy answer for every saved
  reservation. A saved provider deletion can finish after acceptance.
- After an owner removes the App, launched sandboxes stay until their
  runner exits or their saved lifetime ends. Missing job facts do not prove
  that a sandbox is idle. The runtime checks for ended GitHub access before
  and after GitHub retirement. A successful GitHub response does not permit
  early sandbox deletion after removal.
- App removal stops new overflow starts. The last organization removal
  retires each provider key and holds it until its unfinished sandboxes end.
  Exit reports still delete their sandbox, as [Exit callback](#exit-callback)
  says. App removal is rare, and the lifetime and spend limit bound its
  cost, as
  [Removing the App](../../docs/protocol/app-removal.md#removing-the-app)
  explains.
- A new installation clears saved owner removal. Old idle sandboxes can
  then retire through their retained key before the provider connects again.
  Running jobs keep their old delivery and key after a new connection.
- Old idle supply cannot cover a replacement label with the same name.
  Retired labels and removed bindings cannot start an undispatched delivery.
- When a provider or its cost data fails, accepted supply stays reserved.
  New starts can go to another place in the label's order.

## Exit callback

`POST /hooks/overflow/runner-exit` takes the runner-exit report of a sandbox.
`POST /hooks/boat/runner-exit` accepts the same report. Keep that route
until all deliveries saved before the rollout have ended. An unsent
delivery can still hold the old URL in its encrypted launch input.
The route uses no browser session. The guest sends
`GuestToAgent::RunnerExited` as JSON with `Authorization: Bearer TOKEN`.
Each sandbox has its own token, which works only for its own runner.

The route first saves the report as a one-use claim. The claim needs no
workspace lock, so other overflow work of the workspace cannot drop it. The
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
job. The provider ends each sandbox at its lifetime at the latest.

## Live smoke checks

The [live smoke guide](overflow-smoke.md) gives the live Boat checks of a
deployed control plane. It gives their setup, credential file, commands,
local tests, and cleanup.

### Local tests

The [live smoke guide](overflow-smoke.md#local-tests) gives the local tests
of the cleanup delete requests.
