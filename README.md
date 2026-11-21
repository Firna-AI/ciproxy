# ciproxy

ciproxy runs the GitHub Actions jobs of a customer's GitHub
[organizations](docs/protocol/workspaces.md#terms) on the customer's
[own machines](docs/protocol/agent.md#terms) first. When those machines are
busy, it starts runners in the customer's own
[Boat](docs/protocol/overflow.md#the-boat-account) account. A
[workspace](docs/protocol/workspaces.md#terms) holds the organizations, the
machines, and the plan of one customer.

## What ciproxy does

A job uses ciproxy when its `runs-on` line names a ciproxy
[label](docs/protocol/labels.md). ciproxy is the runner provider
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
  When a workspace passes a
  [soft limit](docs/protocol/metering.md#soft-limits), its jobs keep
  running.
- **Open source on the customer's side.** Every part that ciproxy ships
  there is open source under Apache-2.0: the agent, the job image
  definition, the install script, the cache sidecar, and the runner patch.
  [Open source and hosted parts](docs/architecture.md#open-source-and-hosted-parts)
  gives the rule. The hosted [control plane](docs/architecture.md#components)
  and the [dashboard](docs/architecture.md#components) are the paid part.

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

The label picks the [size](docs/protocol/labels.md#terms) of the VM that
runs the job. Every workspace starts with these labels:

| Label           | vCPU | Memory | Free job disk | Slots |
| --------------- | ---- | ------ | ------------- | ----- |
| `ciproxy`       | 2    | 8 GB   | 20 GB         | 1     |
| `ciproxy-4vcpu` | 4    | 16 GB  | 20 GB         | 2     |
| `ciproxy-8vcpu` | 8    | 32 GB  | 60 GB         | 4     |

A [slot](docs/protocol/job-routing.md#slots) is 2 vCPU and 8 GB of an own
machine. The Slots column gives the slots that one job of the label takes
there. Each VM also holds a complete root disk there.
That root includes the image, the free job disk, and an ext4 allowance.
[Labels](docs/protocol/labels.md) gives the rules for labels.
[Job routing](docs/protocol/job-routing.md) gives the rules for slots, the
disk of a job, and where a job runs.

## Get started as a developer

The Rust workspace contains the agent, control plane, and repository gate.
The npm workspace contains the dashboard, site, shared tokens, and mockups.
Run each command from the repository root.

1. Install the Node version in [.nvmrc](.nvmrc).
2. Use rustup to install the Rust toolchain in
   [rust-toolchain.toml](rust-toolchain.toml).
3. Install the npm workspace:

   ```sh
   npm --prefix ts ci
   ```

4. Set up the [Store database](AGENTS.md#commands).
5. Run the repository gate:

   ```sh
   cargo xtask check
   ```

Use `--keep-going` to continue after failures and `--include` or `--exclude`
to select phases, as the [gate guide](xtask/README.md) describes.

The package guides give their commands and main code entry points:

- [ciproxy-agent](crates/ciproxy-agent/README.md): the agent that runs jobs in
  fresh Firecracker VMs on customer machines.
- [ciproxy-protocol](crates/ciproxy-protocol/README.md): shared wire types.
- [ciproxy-control-plane](crates/ciproxy-control-plane/README.md): the hosted
  HTTP runtime and its API, agent, and GitHub webhook routes.
- [xtask](xtask/README.md): the repository gate.
- [@ciproxy/app](ts/app/README.md): the Expo dashboard skeleton.
- [@ciproxy/site](ts/site/README.md): the Astro marketing and Starlight docs.
- [@ciproxy/tokens](ts/tokens/README.md): the shared Graphite design tokens.
- [@ciproxy/mockups](docs/mockups/README.md): the Mokly mockup catalogue.

## Repository layout

- `crates/`: the Rust agent, shared protocol, and hosted control plane.
- `xtask/`: the Rust task runner and repository gate.
- `ts/`: the npm workspace, with `tokens/`, `app/`, `site/`, and `mockups/`.
- `docs/`: the architecture, protocol docs, and mockup sources.
- `plans/`: active plans and follow-up items.
- `AGENTS.md`: the repository rules for agents.
- `rust-toolchain.toml`: the Rust version, with rustfmt and clippy.
- `.cargo/config.toml`: the `cargo xtask` alias.
- `.nvmrc`: the Node version for development and CI.
- `mokly.config.json`: the mockup catalogue and renderer configuration.

## Documentation

The [architecture doc](docs/architecture.md) gives the components of
ciproxy, their responsibilities and boundaries, and the data flow.

The protocol docs hold the requirements and the decisions in force:

- [Workspaces](docs/protocol/workspaces.md): the GitHub organizations of a
  workspace, what the workspace owns, and how an Admin adds an organization
  or takes one out.
- [Labels](docs/protocol/labels.md): the jobs that use ciproxy, the default
  and custom labels, their sizes, and where the jobs of each label may run.
- [Job routing](docs/protocol/job-routing.md): slots, the disk of a job,
  where a job runs, warm runners, and failures.
- [Overflow](docs/protocol/overflow.md): the Boat account, the backend
  contract, overflow runners, the spend limit, and refused sandboxes.
- [Agent](docs/protocol/agent.md): install, the connection, machine
  identity, job VMs, the image, updates, health, drain, and remove.
- [Security](docs/protocol/security.md): sign-in and access, GitHub App
  permissions, repository access, key custody, isolation, and job data.
- [Removing or suspending the App](docs/protocol/app-removal.md): what
  stops, what stays, and what ciproxy deletes when an organization removes
  or suspends the App, and when the last organization of a workspace
  leaves.
- [Cache](docs/protocol/cache.md): the transparent cache, cache scopes,
  trusted write-back, the fallback to the GitHub cache, and Docker disks.
- [Metering](docs/protocol/metering.md): starts, periods, the machine count,
  soft limits, and the savings report of a workspace.
- [Billing](docs/protocol/billing.md): tiers, tier features, add-ons,
  Checkout, the customer portal, tier changes, subscription states, and
  failed payments.

The [mockups README](docs/mockups/README.md) describes the mockup catalogue.
The mockups are part of the spec.

## Licence

Only the directories with their own `LICENSE` file are open source, under
Apache-2.0: `crates/ciproxy-agent`, `crates/ciproxy-protocol`,
`crates/ciproxy-guest`, `crates/ciproxy-cache-sidecar`, `image`, and
`image/runner-patch`. The rest of this repository has no open licence.
