---
title: Set a spend limit
description: Set a monthly spend limit for each overflow provider.
---

Each overflow provider bills you directly. Set a monthly spend limit for
each provider in your workspace. At the limit, jobs skip that provider
and go to the next place in their label order.

## Set or change the limit

Each connection asks for the provider's limit, in Setup or in **Settings**
under **Overflow**. When you change a key or reconnect, the field starts
with the saved limit for that provider.

To change a limit, open **Settings**, then **Overflow**, and find the
provider's panel. Enter a **Monthly spend limit** above zero in US dollars.
Select **Save limit**.

## What the limit counts

- The limit counts only the cost of sandboxes that ciproxy starts for your
  workspace on that provider.
- The month is the calendar month in UTC.
- Until the provider confirms the cost of a start, ciproxy counts a bound.
  It uses the hourly price of the requested size for the full lifetime of
  the sandbox.
- The real runtime cost replaces that part of the bound when the provider
  reports it. When the lifetime passes without a report, the rest of the
  bound stays counted.
- A new start needs enough room under the provider's limit for the counted
  spend, the pending bounds, and the bound of that start.
- When a start has no safe bound, new starts on that provider wait until
  its cost is known.

**Settings**, under **Overflow**, and the **Machines** page show the
confirmed spend and the pending amount separately for each provider.

### Boat start fees

For Boat, the bound also includes the highest published Boat start fee.
When Boat publishes no start fees, ciproxy uses the next-start quote. New
Boat starts wait if neither source gives a fee.

ciproxy saves the fee before the start. The saved fee counts in the UTC
month of the start, even if Boat makes the start free.

## At the limit

- ciproxy holds new starts on that provider. Jobs go to the next place in
  their order, or wait in GitHub when no place can take them.
- Jobs already running on that provider finish. Their cost can take the
  spend above the limit.
- **Jobs** and **Settings**, under **Overflow**, name the provider that
  reached its limit and show when it can take jobs again.
- The provider can take jobs again at the start of the next calendar month
  in UTC. You can also raise its limit to cover the counted spend, pending
  bounds, and a new start.

The savings report counts the cost of the same sandboxes for each provider.
