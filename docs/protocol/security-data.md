# Data and retention

This page states what the control plane receives and stores of a job, and
how long ciproxy keeps each kind of data. [Security](security.md) states
sign-in, access, GitHub permissions, and keys.

## Secrets and job data

- Code and secrets go from GitHub straight to the runner.
- The control plane never stores or logs the code, the secrets, the logs,
  or the cache contents of a job.
- **Decision:** the rule names what the control plane stores and logs.
  Rejected: a rule that it never receives them, because a step name from the
  VM or from GitHub can hold job code.
- The control plane stores the job metadata that the dashboard shows.
- The job metadata is the name, workflow, repository, ref, result, place,
  times, label, [VM size](labels.md#terms) that ran, cache source, and
  [cache counts](cache.md#terms) of a job.
- The job metadata also holds the CPU and memory
  [samples](job-usage.md#terms) and the [steps](job-usage.md#steps) of the
  job.
- The control plane stores no step name that holds job code, as
  [Step names](job-usage.md#step-names) says.
- The guest checks each script name against the actions of the job.
- The control plane also stores what the agent reports about each machine,
  as [Health](agent.md#health) says.
- Job logs stay on GitHub, and the job page links to them.

## Data retention

- ciproxy keeps the job metadata of each finished job for 90 days, on every
  [tier](billing.md#tiers).
- ciproxy keeps the samples and the steps of a job as long as the rest of
  its job metadata, and deletes them with it.
- Each page shows only the jobs of the job history of the tier.
- ciproxy keeps the daily totals of the
  [savings report](metering.md#savings-report) for 90 days.
- ciproxy keeps the last report of each own machine, and the cache hits of
  its last 7 days.
- ciproxy keeps the record of a person only while the person has a role or
  an open request for access.
- After 90 days, ciproxy deletes the job metadata and the daily totals.
- **Decision:** 90 days of job metadata on every tier. Rejected: the window
  of the current tier, 13 months of daily totals, and a setting that makes
  the window shorter. Reason: a tier change then deletes nothing, and an
  upgrade shows the older jobs at once.
- ciproxy deletes the record of a sign-in when the person signs out, or
  within an hour after the sign-in ends.
- ciproxy keeps the configuration of a workspace, such as its labels, its
  settings, and its repository choices, at most while the workspace exists.
- ciproxy deletes that configuration with the data of the workspace.
- [Add an organization](workspaces.md#add-an-organization) says how long
  ciproxy keeps an installation entry.
- ciproxy keeps the billing records of a workspace at most while the
  workspace exists, and deletes them with the data of the workspace.
- The payment record of each move between paid tiers keeps its quote for
  that time.
- ciproxy can delete the other expired quotes, and the references of
  finished Checkout sessions, sooner.
- [Change key and Disconnect](overflow.md#change-key-and-disconnect) says
  how long ciproxy keeps a Boat key after a key change or a Disconnect.
- [Removing the App](app-removal.md#removing-the-app) says when ciproxy
  deletes the data of an organization that removes the App.
- [The last organization](app-removal.md#the-last-organization-of-a-workspace)
  says when ciproxy deletes the data of a workspace whose last organization
  removes the App.
- ciproxy keeps its operational logs, also the request logs of its sites
  and its hosted service, for 30 days.
- The database keeps its 14 most recent daily backups.
- So a deleted record leaves the daily backups when the database has taken
  14 daily backups after the deletion. That is normally within 14 days.
- An operator deletes each manual backup of the database within 14 days, as
  [Delete manual database backups](../../deploy/operations.md#delete-manual-database-backups)
  says.
- Firna Ltd, the provider of ciproxy, keeps each email that it receives
  while the email is useful, or until its sender asks for its deletion.
- Firna Ltd keeps its accounting records, such as invoices, as long as tax
  and accounting law requires.
- **Decision:** each retention period of the privacy policy is a
  requirement here. Rejected: the periods only in the privacy text, and
  reminders that no review checks. Reason: a change of the code or the
  deployment is reviewed against one requirement. Agent call of 22 November
  2026, under the user's delegation.

## Related docs

- [Security](security.md): sign-in, roles, GitHub App permissions,
  repository access, and key custody.
- [Isolation](security-isolation.md): tenant isolation and job isolation.
- [Removing or suspending the App](app-removal.md): what ciproxy revokes,
  keeps, and deletes when an organization removes the App.
- [Job usage](job-usage.md): the samples and the steps of each job, and the
  rule for step names.
- [Billing](billing.md#tiers): the job history of each tier.
- [Metering](metering.md#savings-report): the daily totals of the savings
  report.
- [Agent](agent.md#health): what the agent reports about each machine.
- [Overflow](overflow.md#change-key-and-disconnect): the Boat key after a
  key change or a Disconnect.
- [Workspaces](workspaces.md#add-an-organization): the installation
  entries.
- [Deployment operations](../../deploy/operations.md#delete-manual-database-backups):
  the manual database backups.
