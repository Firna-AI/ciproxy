---
title: Set a spend limit
description: Stop new Boat starts when your workspace reaches its monthly limit.
---

Boat bills you for the sandboxes that ciproxy starts for your workspace. The
spend limit stops new Boat starts when the counted spend of the month
reaches it. Your jobs then run on your machines.

## Set or change the limit

Each connection of a Boat account asks for the limit, in Setup and in
**Settings** under **Overflow**. To change the limit, open **Settings**,
then **Overflow**, and edit **Monthly spend limit**.

The limit is an amount in US dollars above zero. You can change it at any
time.

## What the limit counts

- The spend is the Boat cost of the sandboxes that ciproxy starts for your
  workspace. Other use of your Boat account does not count.
- The month is the calendar month in UTC.
- Until Boat confirms the cost of a start, ciproxy counts a bound for it:
  the published hourly price of the Boat size for the whole lifetime of the
  sandbox, plus the highest published Boat start fee. The lifetime is
  6 hours, or 2 hours on a Boat free trial.
- When Boat publishes no start fees, ciproxy counts the quote of the next
  start instead. When neither gives a fee, new starts wait.
- ciproxy saves that start fee before the start. It stays counted in the
  month of the start, also when Boat made the start free.
- When Boat reports the real cost of the runtime, the real cost takes the
  place of that part of the bound. When the lifetime passes without a
  report, the rest of the bound stays counted.
- A new Boat runner starts only while the limit covers the counted spend,
  the pending bounds, and the bound of the new start. When a start has no
  safe bound, new starts wait until its cost is known.

**Settings**, under **Overflow**, and the **Machines** page show the
confirmed spend and the pending amount apart, against the limit.

## At the limit

- ciproxy starts no new Boat runner. Jobs run on your machines only, and
  wait there for room.
- The Boat jobs that run at that moment finish. So the spend can pass the
  limit by the cost of those jobs.
- **Jobs** and **Settings**, under **Overflow**, say that the spend reached
  the limit, with the date when Boat takes jobs again.
- Boat takes jobs again at the start of the next calendar month in UTC.
- Boat also takes jobs again at once when you raise the limit enough to
  cover the counted spend, the pending bounds, and a new start.

The savings report counts the Boat cost of the same sandboxes.
