# Overflow

This doc states how ciproxy starts [runners](job-routing.md#terms) on Boat
when the [own machines](agent.md#terms) are busy. It also states what a
backend must give ciproxy.

## Terms

- **overflow:** runners that ciproxy starts on a backend because the own
  machines are busy.
- **backend:** a cloud service where ciproxy starts overflow runners.
- **Boat:** Boat sandboxes, the backend.
- **sandbox:** one Boat VM.
- **spend limit:** the monthly limit of Boat spend that the customer sets.

## The Boat account

- **Decision:** overflow runs in the customer's own Boat account, and Boat
  bills the customer. Rejected: ciproxy reselling overflow minutes,
  because the margins are thin, as the
  [open-core decision](../architecture.md#open-source-and-hosted-parts)
  says.
- The customer connects Boat with an API key of its own Boat account.
- The customer does this in Setup or in Settings under Overflow, in the
  [dashboard](../architecture.md#components).
- One Boat account serves the jobs of every organization of the
  [workspace](workspaces.md#terms).
- The customer can skip the Boat step of Setup.
- Without Boat, jobs run on own machines only, as
  [Where a job runs](job-routing.md#where-a-job-runs) says.
- ciproxy checks the key with Boat when the customer saves it.
- ciproxy does not save a key that Boat does not accept, and the key field
  shows an error.
- After the save, the dashboard shows the name of the Boat account and
  never the key, as [Key custody](security.md#key-custody) says.
- The Machines page shows the Boat account and the
  [overflow wait](job-routing.md#where-a-job-runs).
- The customer can change the key or disconnect Boat.

### A rejected key

- When Boat stops accepting the key, jobs run on own machines only until
  the customer connects a new key.
- Settings under Overflow then shows the date when Boat rejected the key.

### Change key and Disconnect

- After the customer changes the key, ciproxy starts new Boat runners
  with the new key.
- Boat jobs that run at that moment finish, and ciproxy deletes the old
  key when the last of their sandboxes ends.
- After Disconnect, ciproxy starts no new Boat runner.
- Boat jobs that run at that moment finish, and ciproxy deletes the key
  when the last of their sandboxes ends.
- [Removing the App](app-removal.md#removing-the-app) says what happens to
  the sandboxes and the key when an organization removes the GitHub App.

## Backend contract

- **Decision:** Boat is the only backend. Reason: Boat has a per-second
  sandbox API.
- A backend gives ciproxy these functions in the account of the customer:
  - It starts a VM of at least a given [size](labels.md#terms) from
    the [image](agent.md#the-image), with the registration of one runner.
  - It bills the customer by the second or finer.
  - It reports the state of each VM.
  - It stops and deletes a VM.
  - It ends a VM by itself, at the latest at a lifetime that ciproxy sets
    when the VM starts. So the end of a VM needs neither the key nor the
    control plane.
  - It lists the VMs that ciproxy started, by a mark that ciproxy sets.
  - It gives the cost of each VM, or the price that ciproxy needs to
    compute it.
  - It accepts a key that the customer creates, and it lets ciproxy check
    the key.
  - It refuses to start a VM with a reason that tells a limit on new VMs,
    a rejected key, and an outage apart.

## Overflow runners

- Each sandbox runs one runner, which takes one job.
- Each sandbox has the cheapest Boat size that covers the label's vCPU,
  memory, and complete VM root allocation.
- That root includes the image, the label's free job disk, and the ext4
  allowance, as [Disk](job-routing.md#disk) says.
- When no size of the account fits, the setting "When nothing fits" of the
  label decides, as
  [A size that nothing can run](labels.md#a-size-that-nothing-can-run)
  says.
- The backend contract includes a call that returns the VM sizes that the
  account can start now. For Boat, this is the size list of its limits
  API. ciproxy reads it again at intervals, because the customer can change
  the Boat plan at any time.
- The job page of a job on Boat names the Boat size that ran it, because
  Boat bills that size.
- So the job page can show a size that is larger than the size of the
  label. With "Run on the largest Boat size", it can also be smaller.
- The largest-size fallback can give less free job disk than the label.
- Its disk must still hold the complete image. Otherwise the job waits.
- New starts and idle runner reuse use this same image and disk rule.
- ciproxy stops and deletes each sandbox when its job ends.
- The ciproxy program in each sandbox tells the control plane when its
  runner exits, with a token that works only for that sandbox.
- The control plane then deletes the sandbox at once, also when GitHub
  reports no job for the runner.
- Each sandbox gets a lifetime of 6 hours, or 2 hours on a Boat free
  trial. Boat ends the sandbox at that time, also during an outage of the
  control plane.
- So a job on Boat runs for at most 6 hours, as a job on a GitHub-hosted
  runner does.
- **Decision:** the sandbox reports the end of its runner, and a lifetime
  set at the start is the backstop. Rejected: a Boat key in each sandbox,
  a lifetime that the control plane renews, a key that ciproxy creates for
  each sandbox, and AWS EC2 as the first backend. Reason: each sandbox
  ends at once when its runner exits, and no sandbox bills past its
  lifetime.
- A Boat runner can stay without a job when an own machine took the
  waiting job first.
- ciproxy stops and deletes the sandbox of such a runner as soon as no
  waiting job needs the runner.
- A Boat runner that takes no job is not a [start](metering.md#starts).
- A job on Boat uses only the [GitHub cache](cache.md#cache-tiers).

### Orphan cleanup

- ciproxy marks each sandbox that it starts.
- ciproxy deletes each marked sandbox that runs no job and whose runner no
  waiting job needs.
- This rule also covers sandboxes whose jobs ended during an outage of the
  [control plane](../architecture.md#components).
- ciproxy never stops or deletes a sandbox that it did not start.

## Spend limit

- The customer sets a monthly spend limit in US dollars in Settings under
  Overflow.
- Each connection of a Boat account asks for the spend limit before Boat
  connects, in Setup and in Settings under Overflow.
- A connection after Boat rejected the key also asks for the spend limit,
  and the field starts with the saved limit.
- The spend limit is an amount above zero. The customer can change it at
  any time, and the MVP has no "no limit" choice.
- **Decision:** each connection of Boat asks for the spend limit, so the
  customer picks the amount in the same step as the key. Rejected: no limit
  until the customer sets one, because a broken workflow could then run up
  a large Boat bill. Rejected: a default limit, and overflow that stays off
  until the customer sets a limit.
- The spend is the Boat cost of the sandboxes that ciproxy starts for the
  workspace.
- The spend does not count other use of the Boat account.
- The spend limit counts over the calendar month in UTC.
- The limit counts confirmed costs and the pending bound of each start
  whose cost Boat has not confirmed.
- The bound uses the published hourly price of the requested Boat size
  for its whole lifetime, plus the highest published start fee saved before
  dispatch.
- If no published start-fee brackets exist, ciproxy saves the next-start
  quote instead.
- If neither source supplies a fee, new starts wait.
- The saved fee stays counted in its UTC start month after runtime
  completion and lifetime expiry, including a start that Boat made free.
- The saved fee is not a provider-confirmed charge.
- **Decision:** count the highest published start fee. Reason: the owner
  said, "let's not over engineer this feature, it's a nice to have, if we
  block a bit early 0.07 early that's really not a problem".
  Rejected: the saved quote, a second quote after the start, and a charged
  fee from Boat.
- The lifetime is 6 hours, or 2 hours on a free trial.
- A start with no clear outcome gets the bound of the size it requested.
- Measured runtime costs replace the same part of the bound, so no cost
  counts twice.
- Real runtime costs replace their bounds, including costs above the bound.
- When the lifetime passes without proof of longer running, the remaining
  bound becomes counted spend. It does not become zero.
- Budget settlement does not confirm a provider cost or a sandbox end.
- New starts continue while the limit covers existing costs and bounds
  together with the prospective start.
- When a start has no safe bound, new starts wait until that cost resolves.
- Settings under Overflow and the Machines page show confirmed spend and
  the pending amount separately against the monthly limit.
- **Decision:** count full-lifetime bounds for unconfirmed costs. Rejected:
  holding every new start until Boat confirms, Admin reconciliation by
  hand, one combined estimate figure, and recovery through a replacement
  key. Reason: Uncounted costs never pass the limit, and overflow keeps
  running while the limit has room.
- At the limit, ciproxy starts no new Boat runner.
- Jobs then run on own machines only and wait there for free
  [slots](job-routing.md#slots) and available [job disk](job-routing.md#terms).
- Boat jobs that run when the spend reaches the limit finish.
- So the spend can pass the limit by the cost of those jobs.
- The Jobs page and Settings under Overflow say that the spend reached the
  limit, with the date when Boat takes jobs again.
- Boat takes jobs again at the start of the next calendar month in UTC.
- Boat also takes jobs again at once when the customer raises the limit
  enough to cover the counted spend, pending bounds, and a new start.

## Refused sandboxes

- When Boat refuses to start a sandbox, for example at a limit of the Boat
  account on new sandboxes, the job keeps waiting.
- The job starts on an own machine as soon as one has the free slots and
  the available job disk for it.
- ciproxy tries again to start a sandbox while the job waits.
- The live queue shows the job as waiting.
- When the reason is a rejected key, jobs run on own machines only until
  the customer connects a new key.

## Related docs

- [Architecture](../architecture.md): the components and the open-core
  decision.
- [Labels](labels.md): the labels, their sizes, and "When nothing fits".
- [Job routing](job-routing.md): slots and where a job runs.
- [Agent](agent.md): own machines and the image.
- [Security](security.md): key custody.
- [Metering](metering.md): starts.
- [Cache](cache.md): the cache of jobs on Boat.
