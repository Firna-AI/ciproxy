# Failures

This page states what happens when a part of ciproxy fails: the control
plane, an own machine, or an overflow provider. It also states the queue
time limit of GitHub. [Job routing](job-routing.md) states the slots, where
each job runs, and the warm runners.

## Control plane outage

- No new runner starts on an own machine or on a provider until the
  control plane is back.
- Jobs without a runner stay queued in GitHub, and they start when the
  control plane is back.
- A control plane outage never stops a running job.

## Own machine offline

- [Connection](agent.md#connection) says when an own machine is offline.
- The dashboard shows no slots for an offline machine.
- ciproxy starts no runner on an offline machine.
- New jobs run on the other own machines and at the other places of their
  order.
- Ready warm runners of an offline machine that still reach GitHub keep
  taking jobs.
- ciproxy does not stop or re-run the running jobs of an offline machine.
- GitHub decides the result of those jobs.
- When the machine is back, the dashboard shows its slots again.

## A provider fails

- While a provider refuses new sandboxes, new jobs skip it and go to the
  next place of their order.
- ciproxy keeps trying the provider for the waiting jobs, as
  [Refused sandboxes](overflow.md#refused-sandboxes) says.
- A missed [ready deadline](overflow-sandbox-end.md#the-ready-deadline)
  does not send a waiting job to the next place.
- A job that no other place of its order can run waits until the provider
  starts a sandbox for it.

## Queue time limit of GitHub

- A queued job stays in GitHub until a runner takes it or the queue time
  limit of GitHub ends it.
- ciproxy does not change that limit.

## Related docs

- [Job routing](job-routing.md#where-a-job-runs): where a job runs, and the
  label order.
- [Agent connection](agent-connection.md): when an own machine is offline.
- [Overflow](overflow.md#refused-sandboxes): refused sandboxes.
- [Data flow](../architecture/data-flow.md#a-control-plane-outage): the
  flow of a control plane outage.
