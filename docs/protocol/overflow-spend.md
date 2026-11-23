# Spend limit

This page states the monthly spend limit of each overflow provider of a
workspace: what it counts, how it bounds the costs that a provider has not
confirmed, and what happens at the limit. [Overflow](overflow.md) states
the provider accounts, the provider contract, and the overflow runners.

## Terms

- [Overflow](overflow.md#terms) defines the overflow provider, Boat, the
  provider account, and the sandbox.
- **spend:** the cost of the sandboxes that ciproxy starts on one provider
  for the [workspace](workspaces.md#terms).
- **spend limit:** the monthly limit of the spend on one provider that the
  customer sets.
- **bound:** the cost that the limit counts for a start while the provider
  has not confirmed the cost of that start.

## The limit of each provider

- Each provider of a workspace has its own monthly spend limit.
- **Decision:** one spend limit for each provider. Rejected: one limit for
  the workspace, and a workspace limit together with a limit for each
  provider. Reason: the user chose this on 23 November 2026.
- The customer sets the spend limit of each connected provider in US
  dollars, in Settings under Overflow.
- Each connection of a provider asks for the spend limit of that provider
  before the provider connects, in Setup and in Settings under Overflow.
- A key change and a connection after a rejected key or after a Disconnect
  also ask for the spend limit. The field starts with the saved limit of
  that provider.
- ciproxy keeps the limit of a provider after a key change and after a
  Disconnect.
- The spend limit is an amount above zero. The customer can change it at
  any time, and the MVP has no "no limit" choice.
- **Decision:** each connection of a provider asks for its spend limit, so
  the customer picks the amount in the same step as the key. Rejected: no
  limit until the customer sets one, because a broken workflow could then
  run up a large bill. Rejected: a default limit, and overflow that stays
  off until the customer sets a limit.

## What the limit counts

- The spend does not count other use of the provider account.
- The spend limit counts over the calendar month in UTC.
- The limit counts confirmed costs and the pending bound of each start
  whose cost the provider has not confirmed.
- The bound uses the hourly price of the requested size, as the provider
  gives it, for the whole [lifetime](overflow-sandbox-end.md#the-lifetime)
  of the sandbox.
- The bound of a Boat start also holds its start fee, as
  [Boat start fees](#boat-start-fees) says.
- A start with no clear outcome gets the bound of the size it requested.
- Measured runtime costs replace the same part of the bound, so no cost
  counts twice.
- Real runtime costs replace their bounds, including costs above the bound.
- When the lifetime passes without proof of longer running, the remaining
  bound becomes counted spend. It does not become zero.
- Budget settlement does not confirm a provider cost or a sandbox end.
- New starts on a provider continue while its limit covers existing costs
  and bounds together with the prospective start.
- When a start has no safe bound, the limit holds new starts on that
  provider until that cost resolves.
- Settings under Overflow and the Machines page show the confirmed spend
  and the pending amount of each provider separately against its monthly
  limit.
- **Decision:** count full-lifetime bounds for unconfirmed costs. Rejected:
  holding every new start until the provider confirms, Admin reconciliation
  by hand, one combined estimate figure, and recovery through a replacement
  key. Reason: Uncounted costs never pass the limit, and overflow keeps
  running while the limit has room.

### Boat start fees

- The bound of a Boat start holds the highest published Boat start fee,
  saved before dispatch.
- If no published start-fee brackets exist, ciproxy saves the next-start
  quote instead.
- If neither source supplies a fee, the limit holds new Boat starts.
- The saved fee stays counted in its UTC start month after runtime
  completion and lifetime expiry, including a start that Boat made free.
- The saved fee is not a provider-confirmed charge.
- **Decision:** count the highest published start fee. Reason: the owner
  said, "let's not over engineer this feature, it's a nice to have, if we
  block a bit early 0.07 early that's really not a problem".
  Rejected: the saved quote, a second quote after the start, and a charged
  fee from Boat.

## At the limit

- At the limit of a provider, the limit holds new starts on that provider,
  and ciproxy starts no new runner there.
- While the limit holds new starts on a provider, jobs skip that provider
  and go to the next place of their order, as
  [Where a job runs](job-routing.md#where-a-job-runs) says.
- The limit of one provider does not stop the other places of an order.
- Jobs that run on that provider when the spend reaches the limit finish.
- So the spend can pass the limit by the cost of those jobs.
- The Jobs page and Settings under Overflow say that the spend on that
  provider reached its limit, with the date when the provider takes jobs
  again.
- The provider takes jobs again at the start of the next calendar month in
  UTC.
- The provider also takes jobs again at once when the customer raises its
  limit enough to cover the counted spend, pending bounds, and a new start.

## Related docs

- [Overflow](overflow.md): the provider accounts, the provider contract,
  and the overflow runners.
- [Sandbox end](overflow-sandbox-end.md#the-lifetime): the lifetime of each
  sandbox.
- [Job routing](job-routing.md#where-a-job-runs): where a job goes when a
  provider cannot take it.
- [Metering](metering.md#savings-report): the overflow costs in the savings
  report.
