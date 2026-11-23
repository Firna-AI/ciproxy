# Overflow

This doc states how a workspace connects overflow providers, and how ciproxy
starts [runners](job-routing.md#terms) on them. It also states what each
provider must give ciproxy, and the rules of Boat, the first provider.
[Spend limit](overflow-spend.md) states the monthly spend limit of each
provider. [Sandbox end](overflow-sandbox-end.md) states when ciproxy
deletes a sandbox.

## Terms

- **overflow provider:** a cloud service where ciproxy starts runners in
  the customer's own account. The short form is "provider".
- **overflow:** the runners that ciproxy starts on overflow providers.
- **Boat:** Boat sandboxes, the first provider.
- **provider account:** the customer's own account at one provider.
- **sandbox:** one VM of a provider.
- **available:** a provider is available when the deployment of the
  [control plane](../architecture.md#components) has the settings of that
  provider.
- **offered:** a provider is offered in a
  [workspace](workspaces.md#terms) after the workspace connected it once.
- The product says "overflow provider", or "provider" where the context is
  clear. It names the brand of a provider, such as Boat, for the things of
  that provider only, such as the Boat key and the Boat wallets.

## Provider accounts

- **Decision:** overflow runs in the customer's own provider accounts, and
  each provider bills the customer. Rejected: ciproxy reselling overflow
  minutes, because the margins are thin, as the
  [open-core decision](../architecture.md#open-source-and-hosted-parts)
  says.
- A workspace can connect several providers, with one account and one key
  for each provider.
- **Decision:** a workspace can connect several providers, and Boat is the
  first. Each label sets its own order of places, such as own machines,
  Boat, then another provider, as [Label order](labels-order.md) says.
  Rejected: one provider at a time, one order for the whole workspace, the
  cheapest size across providers, and one provider for each label. Reason:
  the user chose this on 23 November 2026.
- The customer connects each provider with an API key of its own provider
  account, in Setup or in Settings under Overflow, in the
  [dashboard](../architecture.md#components).
- Setup and Settings list each provider of ciproxy. A provider that is not
  available shows as unavailable right now, and the customer cannot pick it.
- Each provider account serves the jobs of every organization of the
  workspace.
- The customer can skip the provider step of Setup. Without a connected
  provider, jobs skip every provider, as
  [Where a job runs](job-routing.md#where-a-job-runs) says.
- ciproxy checks the key with the provider when the customer saves it.
- ciproxy does not save a key that the provider does not accept, and the
  key field shows an error.
- After the save, the dashboard shows the name of the provider account and
  never the key, as [Key custody](security.md#key-custody) says.
- With no connected provider, Settings under Overflow asks for a provider,
  its key, and its spend limit, as Setup does. Otherwise it shows first the
  overflow switch and the [overflow wait](job-routing.md#the-overflow-wait),
  which hold for every provider, then a panel for each connected or
  available provider.
- The panel of a connected provider shows its account, its spend limit, and
  its spend, with Change key and Disconnect.
- A notice about one provider names that provider, and says when jobs start
  there again.
- The Machines page shows each connected provider with its account and its
  spend, and the overflow wait.

### The first connection of a provider

- When the customer connects a provider that is not offered in the
  workspace yet, ciproxy adds the provider to the end of the
  [order](labels-order.md) of each Linux label.
- The customer can then move the provider or remove it in each label.
- A later connection of an offered provider changes no label order.
- **Decision:** a newly offered provider joins the end of the order of each
  Linux label. Rejected: no label, and a choice of labels in the connect
  step. Reason: the user chose this on 23 November 2026.

### An unavailable provider

- Jobs skip a provider that is not available.
- While a connected provider is not available, Settings under Overflow
  keeps its account, its spend limit, its spend, and Disconnect.
- It shows no Change key then, because no new connection can start.
- When no provider is available, Setup and Settings under Overflow say so,
  and jobs skip every provider.

### A rejected key

- When a provider stops accepting the key, jobs skip that provider until
  the customer connects a new key.
- Settings under Overflow then shows the date when the provider rejected
  the key.

### Change key and Disconnect

- After the customer changes the key of a provider, ciproxy starts new
  runners on that provider with the new key.
- Jobs that run on that provider at that moment finish, and ciproxy deletes
  the old key when the last of their sandboxes ends.
- After Disconnect, ciproxy starts no new runner on that provider.
- Jobs that run on that provider at that moment finish, and ciproxy deletes
  the key when the last of their sandboxes ends.
- A provider that is not connected stays in each label order, and jobs skip
  it. So a key change and a later connection need no label edit.
- [Removing the App](app-removal.md#removing-the-app) says what happens to
  the sandboxes and the keys when an organization removes the GitHub App.

## The Boat account

- The Boat key is an API key of the customer's own Boat account. A Boat
  connection checks the key first. Then the dashboard lists each wallet
  that the key can use, with its name and the likely wallet selected.
- A wallet is the personal Boat account or one Boat organization that pays
  for sandboxes. Each Boat organization has its own wallet.
- The customer chooses the wallet before connecting Boat.
- An organization key offers its own organization wallet.
- **Decision:** the customer chooses an accessible personal or organization
  wallet. Rejected: personal accounts only, and always choosing the default
  wallet. Reason: organization keys and keys with several wallets must work.
- The selected wallet pays for the Boat sandboxes of every organization of
  the workspace.
- ciproxy reads the sizes that a Boat account can start now from the size
  list of the Boat limits API.
- On a Boat free trial, each Boat sandbox gets a lifetime of 2 hours, in
  place of the [lifetime](overflow-sandbox-end.md#the-lifetime) of 6 hours.
- A Boat start can have a start fee, as
  [Boat start fees](overflow-spend.md#boat-start-fees) says.

## Provider contract

- **Decision:** one provider interface serves every provider, with Boat as
  the first provider behind it. The store, the overflow service, the
  dashboard API, and the dashboard use provider-neutral names, and they
  name the provider where a fact belongs to one provider. Rejected: an
  interface inside the overflow service only, and Boat names kept beside a
  provider value. Rejected for now: the provider parts in the shared
  sandbox library, which can follow later. Reason: a rename costs least
  before production, and the user chose this on 23 November 2026.
- Each rule of this doc, of [Spend limit](overflow-spend.md), and of
  [Sandbox end](overflow-sandbox-end.md) holds for every provider, unless
  it names one provider.
- A provider gives ciproxy these functions in the provider account of the
  customer:
  - It starts a VM of at least a given [size](labels.md#terms) from the
    [image](agent.md#the-image), with the registration of one runner.
  - It bills the customer by the second or finer.
  - It reports the state of each VM.
  - It stops and deletes a VM.
  - It ends a VM by itself, at the latest at a lifetime that ciproxy sets
    when the VM starts. So the end of a VM needs neither the key nor the
    control plane.
  - It lists the VMs that ciproxy started, by a mark that ciproxy sets.
  - It gives the cost of each VM, or the price that ciproxy needs to
    compute it.
  - It returns the VM sizes that the account can start now.
  - It accepts a key that the customer creates, and it lets ciproxy check
    the key.
  - It gives the name of the account, whether the account can pay for new
    VMs, and a shorter VM lifetime when the account allows less.
  - It refuses to start a VM with a reason that tells a limit on new VMs, a
    size that the account cannot start, a billing problem, a rejected key,
    and an outage apart. A refusal can give the time when ciproxy can try
    again.
- Each cost of a VM has a bound that ciproxy can compute before the start,
  as [Spend limit](overflow-spend.md#what-the-limit-counts) needs.
- A provider with a cost that has no such bound, such as egress, needs a
  new spend rule before it joins.

## Overflow runners

- Only the jobs of Linux labels run on providers.
- A job of a [macOS label](labels.md#platform) never runs on a provider. It
  waits for a Mac of the workspace, also when overflow is on and the Macs
  are busy.
- **Decision:** macOS jobs never overflow. Rejected: cloud Macs as a
  provider, because cloud Macs that rent by API bill 24 hours at minimum
  under Apple's licence, so per-minute macOS overflow is not possible.
  Reason: this holds from the MVP foundation plan of 3 November 2026.
- The [spend limit](overflow-spend.md) and the overflow wait never apply to
  a macOS job.
- Each sandbox runs one runner, which takes one job.
- Each sandbox has the cheapest size of its provider account that covers
  the label's vCPU, memory, and complete VM root allocation.
- That root includes the image, the label's free job disk, and the ext4
  allowance, as [Disk](job-routing.md#disk) says.
- When no place of the label order fits, the setting "When nothing fits" of
  the label decides, as
  [A size that nothing can run](labels.md#a-size-that-nothing-can-run)
  says.
- ciproxy reads the sizes of each provider account again at intervals,
  because the customer can change the plan of a provider account at any
  time.
- The job page of a job on a provider names the provider and the size that
  ran it, because the provider bills that size.
- So the job page can show a size that is larger than the size of the
  label. With "Run on the largest size", it can also be smaller.
- The largest-size fallback can give less free job disk than the label.
- Its disk must still hold the complete image. Otherwise the fallback does
  not use that provider.
- A new start uses the root of the current image.
- The ciproxy program in each sandbox tells the control plane when its
  runner exits, with a token that works only for that sandbox.
- [Sandbox end](overflow-sandbox-end.md) states when ciproxy stops and
  deletes each sandbox, what the control plane does with that exit report,
  and the ready deadline and the lifetime of each sandbox.
- A runner on a provider that takes no job is not a
  [start](metering.md#starts).
- A job on a provider uses only the [GitHub cache](cache.md#cache-tiers).

## Spend limit

[Spend limit](overflow-spend.md) states the monthly spend limit of each
provider: what it counts, the bounds of costs that the provider has not
confirmed, the Boat start fees, and what happens at the limit.

## Refused sandboxes

- When a provider refuses new sandboxes, for example at a limit of the
  provider account on new sandboxes, the provider takes no job until the
  retry time that it gives.
- When a provider refuses one size, for example because the plan of the
  account does not hold it, only that size stops fitting. A label can then
  use another size of the account that fits.
- Jobs that a provider cannot take go to the next place of their order, as
  [Where a job runs](job-routing.md#where-a-job-runs) says.
- ciproxy tries the provider again while jobs wait.
- The live queue shows a job that no place of its order can take now as
  waiting.
- When the reason is a rejected key, jobs skip that provider until the
  customer connects a new key, as [A rejected key](#a-rejected-key) says.
- When the reason is the billing of the provider account, jobs skip that
  provider until the account can start sandboxes again.

## Related docs

- [Architecture](../architecture.md): the components, the provider
  registry, and the open-core decision.
- [Spend limit](overflow-spend.md): the spend limit of each provider.
- [Sandbox end](overflow-sandbox-end.md): when ciproxy deletes a sandbox,
  the exit report, the ready deadline, and the lifetime.
- [Labels](labels.md): the labels, their sizes, the label order, and "When
  nothing fits".
- [Job routing](job-routing.md): slots and where a job runs.
- [Agent](agent.md): own machines and the image.
- [Security](security.md): key custody.
- [Metering](metering.md): starts.
- [Cache](cache.md): the cache of jobs on providers.
