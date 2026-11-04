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

## Backend contract

- **Decision:** Boat is the only backend. Reason: Boat has a per-second
  sandbox API.
- A backend gives ciproxy these functions in the account of the customer:
  - It starts a VM of at least a given [size](job-routing.md#terms) from
    the [image](agent.md#the-image), with the registration of one runner.
  - It bills the customer by the second or finer.
  - It reports the state of each VM.
  - It stops and deletes a VM.
  - It ends a VM by itself when the runner in the VM exits. So the end of a
    VM needs neither the key nor the control plane.
  - It lists the VMs that ciproxy started, by a mark that ciproxy sets.
  - It gives the cost of each VM, or the price that ciproxy needs to
    compute it.
  - It accepts a key that the customer creates, and it lets ciproxy check
    the key.
  - It refuses to start a VM with a reason that tells a limit on new VMs,
    a rejected key, and an outage apart.

## Overflow runners

- Each sandbox runs one runner, which takes one job.
- Each sandbox is the cheapest VM type of Boat with at least the vCPU,
  memory, and disk of the [label](job-routing.md#labels) of its runner.
- When no size of the account fits, the setting "When nothing fits" of the
  label decides, as [Job routing](job-routing.md#a-size-that-nothing-can-run)
  says.
- The backend contract includes a call that returns the VM sizes that the
  account can start now. For Boat, this is the size list of its limits
  API. ciproxy reads it again at intervals, because the customer can change
  the Boat plan at any time.
- ciproxy stops and deletes each sandbox when its job ends.
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
- **Open:** the spend limit that applies from the moment the customer
  connects Boat. It waits for the decision on the spend limit when Boat
  connects.
- The spend is the Boat cost of the sandboxes that ciproxy starts for the
  [organization](security.md#terms).
- The spend does not count other use of the Boat account.
- The spend limit counts over the calendar month in UTC.
- Settings under Overflow and the Machines page show the spend of the
  month against the limit, for example "$61.20 of $200".
- At the limit, ciproxy starts no new Boat runner.
- Jobs then run on own machines only and wait there for a free
  [slot](job-routing.md#slots).
- Boat jobs that run when the spend reaches the limit finish.
- So the spend can pass the limit by the cost of those jobs.
- The Jobs page and Settings under Overflow say that the spend reached the
  limit, with the date when Boat takes jobs again.
- Boat takes jobs again at the start of the next calendar month in UTC.
- Boat also takes jobs again at once when the customer raises the limit
  above the spend.

## Refused sandboxes

- When Boat refuses to start a sandbox, for example at a limit of the Boat
  account on new sandboxes, the job keeps waiting.
- The job takes the first free slot on an own machine.
- ciproxy tries again to start a sandbox while the job waits.
- The live queue shows the job as waiting.
- When the reason is a rejected key, jobs run on own machines only until
  the customer connects a new key.

## Related docs

- [Architecture](../architecture.md): the components and the open-core
  decision.
- [Job routing](job-routing.md): labels, slots, and where a job runs.
- [Agent](agent.md): own machines and the image.
- [Security](security.md): key custody.
- [Metering](metering.md): starts.
- [Cache](cache.md): the cache of jobs on Boat.
