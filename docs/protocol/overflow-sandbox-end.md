# Sandbox end

This doc states when ciproxy stops and deletes a sandbox of an overflow
provider, what the control plane does with the exit report of its runner,
and how long a waiting job waits for that runner. Its rules hold for every
provider. [Overflow](overflow.md) states the provider accounts, the
provider contract, and the overflow runners.
[Spend limit](overflow-spend.md) states the spend limit of each provider.

## Terms

- [Overflow](overflow.md#terms) defines overflow, the overflow provider,
  Boat, the provider account, and the sandbox.
- **exit report:** the report in which the ciproxy program in a sandbox
  tells the control plane that its runner exited, as
  [Overflow runners](overflow.md#overflow-runners) says.
- **cleanup:** the regular work in which ciproxy deletes the sandboxes that
  it no longer needs.
- **launch:** the moment when the provider accepts the command that starts
  the ciproxy program in a sandbox.
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

## The lifetime

- Each sandbox gets a lifetime of 6 hours, or a shorter one that its
  provider account gives, such as a
  [Boat free trial](overflow.md#the-boat-account).
- The provider ends the sandbox at that time, also during an outage of the
  control plane.
- So a job on a provider runs for at most 6 hours, as a job on a
  GitHub-hosted runner does.
- **Decision:** the sandbox reports the end of its runner, and a lifetime
  set at the start is the backstop. Rejected: a provider key in each
  sandbox, a lifetime that the control plane renews, a key that ciproxy
  creates for each sandbox, and AWS EC2 as the first provider. Reason: each
  sandbox ends soon after its runner exits, and no sandbox bills past its
  lifetime.

## The ready deadline

- Each sandbox gets a ready deadline 10 minutes after ciproxy records its
  launch.
- A runner that takes a job before its ready deadline keeps running.
- When the runner of a sandbox has taken no job by its ready deadline, and
  a waiting job needs that runner, ciproxy replaces the runner.
- ciproxy starts the new runner at once, by the rules of
  [Where a job runs](job-routing.md#where-a-job-runs): at the first place
  of the job's order that can take the job now.
- A missed ready deadline does not send the job to the next place of its
  order.
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
- So a lasting cause of failed starts can repeat paid starts on a provider
  until the [spend limit](overflow-spend.md) of that provider stops new
  starts there.
- **Decision:** ciproxy replaces a runner that takes no job within 10
  minutes of its launch while a job waits for it. Rejected: a failure
  report from the sandbox, because it misses a hang, a lost report, and
  each failure after the ciproxy program removes its stored token.
  Rejected: a ready report from the sandbox with a deadline, because it
  needs a new report and a new image for the same result. Reason: the
  deadline covers each cause of a failed start, also a hang, with a change
  in the control plane only.

## Idle runners

- A runner on a provider can stay without a job when a runner at another
  place took the waiting job first.
- A waiting job of its label needs an idle runner while the sandbox still
  covers the request and the root that the runner started with. A newer
  image with a larger root does not change that.
- A waiting job needs an idle runner that started on the largest size only
  while "Run on the largest size" still applies to its label, as
  [A size that nothing can run](labels.md#a-size-that-nothing-can-run)
  says. The size of the runner must also still be the largest size of its
  provider account. Such a runner keeps the image that it started with.
- ciproxy stops and deletes the sandbox of an idle runner as soon as no
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
- Otherwise, the provider ends the sandbox at its
  [lifetime](#the-lifetime).
- So an idle sandbox of the organization can bill until its lifetime ends.
- The jobs of the organization that run on a provider at the removal
  finish.
- ciproxy deletes each sandbox of the organization whose runner has not
  started.
- **Decision:** after an App removal, ciproxy does not stop idle sandboxes
  early, and they end at their lifetime. Rejected: a new mechanism in the
  sandbox that lets ciproxy stop idle sandboxes early. Reason: App removal
  is rare, and the cost is bounded by the lifetime and the spend limit.

## Related docs

- [Overflow](overflow.md): the provider accounts, the provider contract,
  and the overflow runners.
- [Spend limit](overflow-spend.md): the spend limit of each provider.
- [Job routing](job-routing.md#where-a-job-runs): where a waiting job gets
  its next runner.
- [Removing the App](app-removal.md#removing-the-app): the other effects of
  an App removal.
- [Security](security.md#key-custody): the token of each sandbox.
- [Architecture](../architecture.md#components): the control plane.
