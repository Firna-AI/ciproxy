# ciproxy

ciproxy runs the GitHub Actions jobs of an
[organization](docs/protocol/security.md#terms) on the customer's
[own machines](docs/protocol/agent.md#terms) first. When those machines are
busy, it starts runners in the customer's own
[Boat](docs/protocol/overflow.md#the-boat-account) account.

## What ciproxy does

A job uses ciproxy when its `runs-on` line names a ciproxy
[label](docs/protocol/job-routing.md#labels). ciproxy is the runner provider
for its labels. It starts each [runner](docs/protocol/job-routing.md#terms)
on a free own machine, or else on Boat.

- **Own machines first.** The [agent](docs/protocol/agent.md#terms) runs on
  each own machine. Each own machine keeps
  [warm runners](docs/protocol/job-routing.md#warm-runners) ready, so a job
  can start at once.
- **Overflow to Boat.** When the own machines are busy, ciproxy starts
  [overflow](docs/protocol/overflow.md#overflow-runners) runners in the
  customer's own Boat account. Boat bills the customer. The customer sets a
  monthly [spend limit](docs/protocol/overflow.md#spend-limit) for Boat.
- **A fresh VM for each job.** Each job runs in a new
  [job VM](docs/protocol/agent.md#job-vms) or Boat
  [sandbox](docs/protocol/overflow.md#terms) that no other job used.
- **A transparent cache.** `actions/cache` works without workflow edits. On
  an own machine, it reads and writes through the
  [machine cache](docs/protocol/cache.md#transparent-cache).
- **Soft limits.** Each [tier](docs/protocol/billing.md#tiers) includes a
  number of [starts](docs/protocol/metering.md#starts) and own machines.
  When an organization passes a
  [soft limit](docs/protocol/metering.md#soft-limits), its jobs keep
  running.
- **An open-source agent.** The agent is open source under Apache-2.0. The
  hosted [control plane](docs/architecture.md#components) and the
  [dashboard](docs/architecture.md#components) are the paid part.

ciproxy runs x64 Linux jobs of
[private repositories](docs/protocol/security.md#repository-access) only.

## Use ciproxy in a workflow

To move a job to ciproxy, change one line. Replace `ubuntu-latest` in its
`runs-on` line with a ciproxy label:

```diff
 jobs:
   test:
-    runs-on: ubuntu-latest
+    runs-on: ciproxy
```

The label picks the [size](docs/protocol/job-routing.md#labels) of the VM
that runs the job. Every organization starts with these labels:

| Label | vCPU | Memory | Disk | Slots |
| --- | --- | --- | --- | --- |
| `ciproxy` | 2 | 8 GB | 60 GB | 1 |
| `ciproxy-4vcpu` | 4 | 16 GB | 80 GB | 2 |
| `ciproxy-8vcpu` | 8 | 32 GB | 120 GB | 4 |

A [slot](docs/protocol/job-routing.md#slots) is 2 vCPU and 8 GB of an own
machine. The Slots column gives the slots that one job of the label takes
there. [Job routing](docs/protocol/job-routing.md) gives the rules for
labels, slots, and where a job runs.

## Get started as a developer

The mockup catalogue is the only part of the repository that runs. It shows
every screen of the dashboard and the public site. Run each command from the
repository root.

1. Install Node 22.15 or newer. Do not use Node 23 or Node 24.14 to 24.18.
2. Install the npm workspace:

   ```sh
   npm --prefix ts ci
   ```

3. Serve the catalogue with live reload:

   ```sh
   npm --prefix ts run mockups:serve
   ```

4. Build the catalogue and run its checks:

   ```sh
   npm --prefix ts run mockups:check
   ```

The [mockups README](docs/mockups/README.md) gives the other commands and
the rules of the catalogue.

## Repository layout

- `docs/architecture.md`: the components of ciproxy and the data flow
  between them.
- `docs/protocol/`: the protocol docs, with the requirements and the
  decisions in force.
- `docs/mockups/`: the mockup catalogue. Its sources are under
  `docs/mockups/src`.
- `ts/`: the npm workspace of the mockups.
- `mokly.config.json`: the configuration of Mokly, which builds the mockup
  catalogue.

## Documentation

The [architecture doc](docs/architecture.md) gives the components of
ciproxy, their responsibilities and boundaries, and the data flow.

The protocol docs hold the requirements and the decisions in force:

- [Job routing](docs/protocol/job-routing.md): labels, sizes, slots, where a
  job runs, warm runners, and failures.
- [Overflow](docs/protocol/overflow.md): the Boat account, the backend
  contract, overflow runners, the spend limit, and refused sandboxes.
- [Agent](docs/protocol/agent.md): install, the connection, machine
  identity, job VMs, the image, updates, health, drain, and remove.
- [Security](docs/protocol/security.md): sign-in and access, GitHub App
  permissions, repository access, key custody, isolation, and job data.
- [Cache](docs/protocol/cache.md): the transparent cache, cache scopes,
  trusted write-back, the fallback to the GitHub cache, and Docker disks.
- [Metering](docs/protocol/metering.md): starts, periods, the machine count,
  soft limits, and the savings report.
- [Billing](docs/protocol/billing.md): tiers, add-ons, Checkout, the
  customer portal, tier changes, subscription states, and failed payments.

The [mockups README](docs/mockups/README.md) describes the mockup catalogue.
The mockups are part of the spec.
