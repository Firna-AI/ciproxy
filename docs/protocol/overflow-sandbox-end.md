# Sandbox end

This doc states when ciproxy stops and deletes a Boat sandbox, what the
control plane does with the exit report of its runner, and how long a
waiting job waits for that runner.
[Overflow](overflow.md) states the Boat account, the backend contract, the
overflow runners, and the spend limit. [Job usage](job-usage.md) states the
samples and the steps that the reports of a sandbox carry.

## Terms

- [Overflow](overflow.md#terms) defines overflow, the backend, Boat, the
  sandbox, and the spend limit.
- **exit report:** the report in which the ciproxy program in a sandbox
  tells the control plane that its runner exited, as
  [Overflow runners](overflow.md#overflow-runners) says.
- **sample report:** a report of the samples and the steps of a running Boat
  job, as [Sample reports](#sample-reports) says.
- **cleanup:** the regular work in which ciproxy deletes the sandboxes that
  it no longer needs.
- **launch:** the moment when Boat accepts the command that starts the
  ciproxy program in a sandbox.
- **ready deadline:** the time by which the runner of a sandbox must take a
  job, as [The ready deadline](#the-ready-deadline) says.

## The end of a job

- ciproxy stops and deletes each sandbox when its job ends.
- The control plane also deletes a sandbox when it saves the
  [exit report](#the-exit-report) of its runner. This holds also when
  GitHub reports no job for the runner.
- A delete that other overflow work of the workspace holds up, or that
  fails, runs at a later cleanup.

## The exit report

- The control plane accepts one exit report for each sandbox, and only
  while it has not recorded the end of that sandbox.
- The exit report carries the final [samples](job-usage.md#terms) and
  [steps](job-usage.md#steps) of the job.
- The control plane saves an accepted report before it deletes the
  sandbox.
- A success answer means only that the control plane saved the report.
- After the save, the control plane deletes the sandbox in the same
  request.
- When other overflow work of the workspace runs, such as a start or a
  cleanup, that delete waits up to 5 seconds for it.
- When that wait ends first, a later cleanup deletes the sandbox.
- Later overflow work of the workspace never undoes a saved report.
- A recorded sandbox end keeps its first time.
- **Decision:** the control plane saves the report before it waits for
  other overflow work, and a later cleanup deletes the sandbox when that
  wait ends first. Rejected: a longer wait before the save, retries from
  the sandbox, and a delay of the job-end delete, because each can still
  lose a report. Rejected: no wait after the save, because fewer deletes
  then run in the request. Reason: a saved report takes effect whatever
  other work runs, and the sandbox sends it only once.

### Sample reports

- While the job runs, the ciproxy program in the sandbox sends a sample
  report every 30 seconds.
- It sends each sample report with the token of its sandbox, as it sends
  the exit report.
- **Decision:** each sample report holds all the samples and the steps of
  the job so far. Rejected: only the new points, because the points merge as
  the samples grow. Reason: a report then never depends on an earlier one.
- So a long job sends up to about 100 KB every 30 seconds.
- The control plane accepts a sample report only while it has not recorded
  the end of that sandbox.
- It accepts at most one sample report for each sandbox in 10 seconds.
- It checks each sample report as [Checks](job-usage.md#checks) says.
- Each accepted sample report replaces the samples and the steps that the
  control plane kept for the job.
- The exit report replaces them with the final samples and steps.
- No sample report replaces the final samples and steps of an accepted exit
  report, also when it arrives after that report.
- A sample report is not an exit report, so the control plane still accepts
  only one exit report for each sandbox.
- These limits apply because job code can read the token of the sandbox, as
  [Key custody](security.md#key-custody) says.

## The lifetime

- Each sandbox gets a lifetime of 6 hours, or 2 hours on a Boat free
  trial. Boat ends the sandbox at that time, also during an outage of the
  control plane.
- So a job on Boat runs for at most 6 hours, as a job on a GitHub-hosted
  runner does.
- **Decision:** the sandbox reports the end of its runner, and a lifetime
  set at the start is the backstop. Rejected: a Boat key in each sandbox,
  a lifetime that the control plane renews, a key that ciproxy creates for
  each sandbox, and AWS EC2 as the first backend. Reason: each sandbox
  ends soon after its runner exits, and no sandbox bills past its
  lifetime.

## The ready deadline

- Each sandbox gets a ready deadline 10 minutes after ciproxy records its
  launch.
- A runner that takes a job before its ready deadline keeps running.
- When the runner of a sandbox has taken no job by its ready deadline, and
  a waiting job needs that runner, ciproxy replaces the runner.
- ciproxy starts the new runner at once, by the rules of
  [Where a job runs](job-routing.md#where-a-job-runs): on an own machine
  first, and otherwise on Boat within the
  [spend limit](overflow.md#spend-limit).
- ciproxy first removes the GitHub registration of the old runner, and then
  deletes its sandbox.
- When GitHub gave the old runner a job before that removal, ciproxy keeps
  its sandbox, and the job runs.
- A runner that no waiting job needs follows
  [Idle runners](#idle-runners), also before its ready deadline.
- An App removal changes this rule, as
  [After an App removal](#after-an-app-removal) says.
- So ciproxy also replaces a slow but healthy runner that has taken no job
  by its ready deadline.
- So a lasting cause of failed starts can repeat paid Boat starts until the
  spend limit stops new starts.
- **Decision:** ciproxy replaces a runner that takes no job within 10
  minutes of its launch while a job waits for it. Rejected: a failure
  report from the sandbox, because it misses a hang, a lost report, and
  each failure after the ciproxy program removes its stored token.
  Rejected: a ready report from the sandbox with a deadline, because it
  needs a new report and a new image for the same result. Reason: the
  deadline covers each cause of a failed start, also a hang, with a change
  in the control plane only.

## Idle runners

- A Boat runner can stay without a job when an own machine took the
  waiting job first.
- ciproxy stops and deletes the sandbox of such a runner as soon as no
  waiting job needs the runner.
- An App removal changes this rule, as
  [After an App removal](#after-an-app-removal) says.

## Orphan cleanup

- ciproxy marks each sandbox that it starts.
- ciproxy deletes each marked sandbox that runs no job and whose runner no
  waiting job needs.
- This rule also covers sandboxes whose jobs ended during an outage of the
  [control plane](../architecture.md#components).
- An App removal changes this rule, as
  [After an App removal](#after-an-app-removal) says.
- ciproxy never stops or deletes a sandbox that it did not start.

## After an App removal

- These rules apply to the sandboxes of an
  [organization](workspaces.md#terms) that removed the GitHub App.
- A new install that joins the organization to the workspace that it left
  ends these rules.
- [Removing the App](app-removal.md#removing-the-app) states the other
  effects of a removal.
- ciproxy does not stop a sandbox of the organization whose runner started,
  also when the runner has no job.
- ciproxy still deletes such a sandbox when its runner reports its exit, as
  [The exit report](#the-exit-report) says.
- Otherwise, Boat ends the sandbox at its lifetime of 6 hours, or 2 hours on
  a Boat free trial.
- So an idle sandbox of the organization can bill until its lifetime ends.
- Boat jobs of the organization that run at the removal finish.
- ciproxy deletes each sandbox of the organization whose runner has not
  started.
- **Decision:** after an App removal, ciproxy does not stop idle sandboxes
  early, and they end at their lifetime. Rejected: a new mechanism in the
  sandbox that lets ciproxy stop idle sandboxes early. Reason: App removal
  is rare, and the cost is bounded by the lifetime and the spend limit.

## Related docs

- [Overflow](overflow.md): the Boat account, the backend contract, the
  overflow runners, and the spend limit.
- [Job routing](job-routing.md#where-a-job-runs): where a waiting job gets
  its next runner.
- [Removing the App](app-removal.md#removing-the-app): the other effects of
  an App removal.
- [Security](security.md#key-custody): the token of each sandbox.
- [Job usage](job-usage.md): the samples and the steps that the reports
  carry.
- [Architecture](../architecture.md#components): the control plane.
