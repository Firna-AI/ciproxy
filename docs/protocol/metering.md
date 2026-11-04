# Metering

This doc states how ciproxy counts the use of an
[organization](security.md#terms): its starts, its periods, and its
[own machines](agent.md#terms). It also states the soft limits and the
savings report. [Billing](billing.md#tiers) gives the price and the numbers
of each tier.

## Terms

- **start:** one job that a ciproxy [runner](job-routing.md#terms) takes.
- **period:** the time over which ciproxy counts the starts of an
  organization.
- **soft limit:** a limit that the
  [dashboard](../architecture.md#components) shows but that never stops a
  job.

## Starts

- A start is one job that a ciproxy runner takes, on an own machine or on
  [Boat](overflow.md#terms), whatever its result.
- A job becomes a start at the moment that a runner takes it.
- A re-run of a job is a new start.
- A [warm runner](job-routing.md#warm-runners) that takes no job is not a
  start.
- A Boat runner that takes no job is not a start.
- A job that no runner takes is not a start, for example a job that someone
  cancels while it waits.
- The job page of each start says "Counts as 1 start".
- **Decision:** the price counts starts. Rejected: a price per own machine,
  per vCPU, or per job-minute, a share of the savings, and a flat price.
  Reason: starts grow with AI coding agents and pull requests, which drive
  CI load, and the machine count is simple to check.

## Periods

- A paid tier counts starts over its Stripe billing period, for example
  4 November to 3 December.
- Free counts starts over periods of one month from the day that the
  organization starts on Free, for example 1 September to 30 September.
- The days and the dates of every period are in UTC.
- An organization is in one period only at any moment.
- A start counts in the period in which a runner took the job.
- Plan and usage shows the dates of the current period.
- A move from Free to a paid tier ends the Free period when Stripe confirms
  the payment, and a new period starts then.
- The starts of the Free period stay in it, and the new period counts from
  zero.
- A move between paid tiers keeps the period.
- The starts of the period so far count against the new tier.
- When a [subscription](billing.md#subscription-states) ends, a new Free
  period starts that day.

## Own machines

- The machine count of an organization is the number of own machines that
  are registered to it.
- The count is the current number, not the highest number of the period.
- The count includes every registered machine, also one that is offline or
  [draining](agent.md#drain-and-remove).
- A machine joins the count when it registers.
- A machine leaves the count at once when the customer removes it.

## Soft limits

- The included starts and the included own machines of an organization are
  the numbers of its tier plus its [add-ons](billing.md#add-ons).
- These two numbers are the soft limits of the organization.
- An organization passes a soft limit when the starts of the period or the
  machine count go above it.
- A soft limit never stops a job and never keeps a job from starting.
- Jobs keep running on every own machine, also when the machine count is
  above the included own machines.
- Passing a soft limit adds no charge.
- While an organization is above a soft limit, every page of the
  organization shows one banner.
- The banner says that jobs keep running, and it gives the offer.
- **Decision:** tier limits are soft. Rejected: a hard limit, because a
  pricing limit must never stop the CI of a customer.

### The use and the offer

- The expected starts are the starts of the period so far, divided by the
  days so far, times the days of the period.
- The days so far include the current day.
- The use of an organization is its expected starts and its machine count.
- A tier with its add-ons covers the use when it includes at least the
  expected starts and at least the machine count.
- The offer is the cheapest choice of a paid tier and add-on quantities
  that covers the use.
- On Free, the offer is the paid tier of that choice, with the add-ons that
  it needs.
- On a paid tier, when that choice keeps the tier, the offer is the
  smallest add-on quantity that covers the use.
- The Plans page marks the tier of the offer with "Covers your use", unless
  it is the current tier.
- When the customer chooses the tier of the offer, the checkout review
  starts with the add-on quantities of the offer.

### What Plan and usage shows

- Plan and usage shows the starts of the period against the included
  starts.
- While the starts are above the included starts, it shows how many starts
  are over and that jobs keep running.
- Otherwise it shows the expected starts.
- It shows the machine count against the included own machines.
- While the machine count is above the included own machines, it shows how
  many machines are over and that every machine runs jobs.
- While the starts are below the included starts and the expected starts
  are above them, Plan and usage shows a notice.
- The notice gives the day on which the starts pass the included starts at
  the current pace, and it gives the offer.

## Savings report

- The Savings page shows the savings report of the organization.
- The report compares the cost of the jobs with ciproxy with the cost of
  the same jobs on Blacksmith and on GitHub-hosted runners.
- **Open:** the prices of the comparison. The mockups give list prices per
  minute for the same runner sizes. It waits for item 9, the price basis of
  the cost claim.
- The report covers one of three ranges: the last 30 days, this calendar
  month, or the last calendar month.
- It opens on the last 30 days.
- The days of the report are UTC days.
- The cost with ciproxy is the cost of the own machines, plus the Boat
  cost, plus the price of the tier and its add-ons.
- The customer enters the monthly cost of each own machine on its machine
  page.
- The report counts the monthly cost of an own machine for the days of the
  range on which the machine was registered.
- The report counts the monthly price of the tier and its add-ons for the
  days of the range.
- The Boat cost counts the same [sandboxes](overflow.md#terms) as the
  [spend limit](overflow.md#spend-limit), for the days of the range.
- The savings against an alternative are its cost minus the cost with
  ciproxy.
- The report leads with the savings against the cheaper alternative.
- When the cost with ciproxy is higher than the cost of an alternative,
  the report shows how much higher it is.
- When an own machine has no monthly cost, the report shows no savings
  figure and asks for the missing costs.
- The report then still shows the costs of the alternatives, the Boat cost,
  and the tier price.
- The report shows the starts and the minutes on own machines and on Boat.
- It shows the starts of each day, on own machines and on Boat.
- On every tier, ciproxy keeps the daily totals of at least the current
  calendar month, the last calendar month, and the last 30 days.
- The daily totals stay also when the [job history](billing.md#tiers) of
  the tier is shorter.

## Related docs

- [Architecture](../architecture.md#components): the components that count
  the use and show it.
- [Billing](billing.md#tiers): the tiers, their numbers, and the add-ons.
- [Job routing](job-routing.md#where-a-job-runs): where a runner takes each
  job.
- [Agent](agent.md#machine-identity): how an own machine registers.
- [Overflow](overflow.md#spend-limit): the Boat spend of the organization.
