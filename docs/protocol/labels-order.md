# Label order

This page states the order of each label: the places where its jobs run,
in order, and how the customer changes them. [Labels](labels.md) states the
labels and their sizes.
[Where a job runs](job-routing.md#where-a-job-runs) states how a job moves
along its order.

## Terms

- [Labels](labels.md#terms) defines the label, the label order, and the
  place.
- [Overflow](overflow.md#terms) defines the overflow provider, and when a
  provider is available or offered.

## The order of a label

- Each label has an order: the places where its jobs run, in order.
- A place is own machines or one provider. An order holds each place at
  most once.
- A job goes to the next place of its order when the current place cannot
  take it now, as [Where a job runs](job-routing.md#where-a-job-runs) says.
- So an order can list a provider before own machines, and it can leave
  own machines out.
- [Provider accounts](overflow.md#provider-accounts) gives the decision on
  several providers and the order of each label.
- An order can be empty. The jobs of a label with an empty order wait in
  GitHub, and the dialog says so.
- The order of a macOS label holds own machines only, as
  [Platform](labels.md#platform) says.
- A provider that is not connected or not
  [available](overflow.md#terms) stays in each order, and jobs skip it.
- Such a provider counts as a place that fits only for the wait reason, as
  [A size that nothing can run](labels.md#a-size-that-nothing-can-run)
  says.
- Own machines keep warm runners for a label only while they are its first
  place, as [Warm runners](job-routing.md#warm-runners) says.

## Changes to the order

- The dialog of a Linux label shows its order. There the customer can move
  a place, remove a place, and add a place.
- "Add a place" offers own machines and each
  [offered](overflow.md#terms) provider that the order does not hold.
- In the dialog, each place says what it does with a job of the label's
  size: how long a job waits for own machines, the provider size that runs
  it, or why jobs skip the place.
- A provider place says first when overflow is off, or when the provider is
  not connected or not available. Only then does it say whether a size of
  the provider fits.
- While the size of a label [fits](job-routing.md#fit) no own machine, jobs
  skip own machines. The dialog says so before the customer saves the
  label.
- When an own machine that fits the size connects, the jobs of the label
  can run on own machines again, at their place in the order.

## New labels and new providers

- A new Linux label gets own machines first, then each offered provider in
  the order of their first connections. This holds also for a default
  label.
- A provider that the workspace connects for the first time joins the end
  of the order of each Linux label, as
  [The first connection of a provider](overflow.md#the-first-connection-of-a-provider)
  says.

## Related docs

- [Labels](labels.md): the labels, their sizes, their platform, and "When
  nothing fits".
- [Job routing](job-routing.md#where-a-job-runs): how a job moves along its
  order, and the overflow wait.
- [Overflow](overflow.md): the provider accounts and the first connection
  of a provider.
