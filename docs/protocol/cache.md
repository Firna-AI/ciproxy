# Cache

ciproxy gives each job the cache of GitHub Actions without workflow edits.
This doc states the cache rules for jobs on
[own machines](agent.md#terms) and on [Boat](overflow.md#the-boat-account).

## Terms

- **machine cache:** the cache disk of an own machine.
- **GitHub cache:** the cache service of GitHub Actions that
  `actions/cache` uses.
- **cache scope:** the rule that says which jobs can restore a cache entry.
- **cache counts:** how many entries a job restored, and the space of these
  entries.
- **Docker disk:** a disk for one repository on one own machine that keeps
  Docker data between the jobs of the repository.
- The [tiers](billing.md#tiers) call the Docker disk feature "sticky Docker
  disks".

## Transparent cache

- `actions/cache` and other actions that use the GitHub cache work on
  ciproxy [runners](job-routing.md#terms) without workflow edits.
- On an own machine, a job restores and saves its cache entries through the
  machine cache.
- Artifacts and logs go to GitHub, as on GitHub-hosted runners.
- **Decision:** the cache is transparent to workflows. Rejected: a cache
  that each workflow names in its steps, because it needs workflow edits.

## Cache scopes

- A job restores entries by the same keys, versions, and cache scopes as
  in the GitHub cache.
- A job never restores an entry of another repository.
- A job with a checked ref restores its signed readable scopes and the
  default branch, with its own scope first and the default branch last.
- Signed readable scopes include the base branch of a pull request.
- The machine keeps GitHub's scope keys unchanged, also for tags.
- A job without a checked ref restores only default-branch entries locally.
- Its other restores go through to the GitHub cache.
- Its saves and GitHub fills use a fresh private scope for each session.
- No job restores that private scope, including the job that created it.
- Its permitted saves also go through to the GitHub cache.
- A missing or invalid token never fails the job.
- The machine report counts jobs without a checked ref by reason.
- **Decision:** an unknown ref keeps default-branch local restores and
  private saves. Rejected: GitHub-only use, because a token format change
  would remove every local hit, and later promotion, because it adds work
  for a rare case.
- The machine cache keeps these rules for every job on its machine.
- The machine cache keeps an entry after the GitHub cache evicts it.
- Later jobs on the machine can still restore such an entry.
- The machine cache also keeps its copy of an entry that a person deletes
  on GitHub.
- "Clear cache" in the [dashboard](../architecture.md#components) deletes
  every entry of the machine cache of one machine.
- The dashboard shows "Clear cache" next to the cache use of that machine.
- "Clear cache" never deletes an entry of the GitHub cache.
- "Clear cache" never deletes a [Docker disk](#docker-disks) of the
  machine.
- **Decision:** the machine cache keeps entries after the GitHub cache
  evicts them, and "Clear cache" removes a bad entry. Rejected: restoring
  only the entries that the GitHub cache still has, because the machine
  cache could then serve no more entries than the GitHub cache holds.
- **Decision:** "Clear cache" acts on one machine. Reason: it sits with the
  cache use of that machine.

## Trusted write-back

- Only a job on the default branch writes machine-cache entries that every
  branch can restore.
- Only these events can write shared machine-cache entries: `push`,
  `workflow_dispatch`, `repository_dispatch`, `delete`, `registry_package`,
  `page_build`, and `schedule`.
- A job of another event cannot publish a shared machine-cache entry, also
  when it restores an entry from GitHub.
- A pull request can write machine-cache entries within its trusted
  `refs/pull/N/merge` ref only.
- Another branch or pull request cannot restore those entries.
- A job with a foreign head repository cannot write the default scope.
- The cache scope of a save comes from GitHub, not from the job, so the
  code of a job cannot choose it.
- The machine checks the runtime token outside the job VM at registration.
- It accepts only RS256 with a key ID present in GitHub's public key set.
- It checks the exact GitHub Actions issuer and the active validity interval.
- It checks the assigned repository, run, runner, and supplied job identity.
- Exactly one signed access-control scope with the write bit gives the ref.
- Missing, invalid, absent-write, and multiple-write results are unknown.
- A shared write requires the checked ref to equal
  `refs/heads/<default branch>` exactly.
- A tag scope such as `refs/heads/refs/tags/main` cannot match that branch.
- The control plane reads the event, head repository, and default branch
  from the GitHub API, as
  [GitHub App permissions](security.md#github-app-permissions) says.
- The machine refreshes GitHub's public keys every five minutes.
- It also permits one rate-limited refresh for an unknown key ID.
- Failed fetches have the same refresh limits.
- **Decision:** a checked runtime-token scope supplies the full ref.
  Rejected: the run API alone and reusable workflow refs, because they
  cannot identify every full job ref. Rejected: a hook, a test cache entry,
  a push webhook, and OIDC, because signed scopes need none of their extra
  trust, requests, permissions, or workflow changes.
- Direct GitHub writes enabled with `cache-mode: write` follow GitHub's
  permission rules, as on GitHub-hosted runners.
- **Decision:** the machine cache follows
  [GitHub's trusted-event rule][trusted-events] without workflow edits.
  Rejected: required read-only rights that need workflow edits, and a large
  runtime-token broker that blocks GitHub's write opt-in.
- **Decision:** only jobs on the default branch write machine-cache entries
  for every branch. Rejected: shared writes from every branch, because a
  pull request could then poison the cache.

[trusted-events]: https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#cache-access-for-low-trust-workflow-triggers

## Cache tiers

- On an own machine, the machine cache gives the contents of a restored
  entry when it has them.
- Otherwise the contents come from the GitHub cache, and the machine cache
  keeps them.
- A save goes to the machine cache and through to the GitHub cache, so
  other own machines and jobs on Boat can restore it.
- A job on Boat uses the GitHub cache only.
- The job page shows the cache source of each job: "This machine", "The
  GitHub cache", or "This machine and the GitHub cache".
- On an own machine, the cache source names the caches that gave the
  contents of the entries that the job restored.
- A job that restored entries from both caches shows "This machine and the
  GitHub cache".
- **Decision:** a job that restored from both caches shows "This machine
  and the GitHub cache". Rejected: one of the two sources for such a job,
  because it is false for some of its entries. Reason: the cache source is
  then true for every job.
- A job on Boat has the cache source "The GitHub cache".
- A job that restored no entry shows "No entries restored". Reason: it is
  true for every such job.
- The job page shows the cache counts of a job only when the machine cache
  handled the restores of the job, for example "3 restored · 2.1 GB".
- A job on Boat, and a job that fell back to the GitHub cache, show only
  the cache source.
- **Decision:** only a job whose restores the machine cache handled shows
  cache counts. Rejected: counts that the image sends with a token, because
  job code could fake them. Rejected: counts through the Boat API, and no
  counts on any job. Reason: the counts need no data from a sandbox, and
  the rule holds for every later backend.
- The Machines page and the machine page show the used space and the total
  space of the machine cache.
- The total space of the machine cache changes as job VMs start and end,
  as [Disk](job-routing.md#disk) says.
- They also show the cache hits of the last 7 days.
- The cache hits of a machine are the share of its cache restores of the
  last 7 days that the machine cache served.
- On the Machines page, the cache hits cover the cache restores of all own
  machines.

## Fallback to the GitHub cache

- A failure of the machine cache never fails a job.
- The job then uses the GitHub cache directly, or it sees a cache miss.

## Docker disks

- A Docker disk keeps the Docker data of one repository on one own machine
  between the jobs of the repository.
- A Docker disk attaches to the [job VM](agent.md#job-vms) after the job
  starts, because ciproxy learns the repository of a job only then.
- One job at a time keeps its changes to a Docker disk: the first job of a
  trusted event on the default branch that uses the disk.
- The trusted events are those of
  [Trusted write-back](#trusted-write-back): `push`, `workflow_dispatch`,
  `repository_dispatch`, `delete`, `registry_package`, `page_build`, and
  `schedule`.
- Each other job of the repository works on a copy of the disk, also each
  job that does not run on the default branch, and each job of another
  event.
- ciproxy discards a copy when its job ends.
- The Docker disk uses the same machine-checked ref and trusted-event rule
  as the machine cache.
- A job without a checked ref gets a disposable copy of the Docker disk.
- ciproxy discards that copy when the job ends.
- The Docker disk of one repository never attaches to a job of another
  repository.
- A job on Boat gets no Docker disk.
- An attached Docker disk or copy reserves its data plus the free disk of
  the label of its job.
- It takes that reservation only from the spare disk of the machine, as
  [Disk](job-routing.md#disk) says.
- It keeps the reservation until its job ends.
- It never shrinks or detaches during its job.
- When the spare disk has no room for the reservation, or the disk cannot
  attach, the job runs without its Docker disk.
- Docker then keeps its data on the root disk of the job VM.
- So a machine whose disk limits its jobs gets few or no Docker disks.
- **Decision:** an attached Docker disk reserves its data plus the free disk
  of its label. Rejected: a Docker budget, because the 250 GB minimum then
  fits no default job. Reason: Docker then gets the free disk of the label,
  as it does on the root disk.
- The machine page shows how many repositories have a Docker disk on the
  machine, and the space that these disks take.
- Only Team and Business include Docker disks, as
  [Tier features](billing.md#tier-features) says.
- A job of a workspace on Free gets no Docker disk.
- **Decision:** only a job of a trusted event on the default branch keeps
  its changes to a Docker disk, as for the machine cache. Rejected: any
  event on the default branch, because a person outside the repository
  can start some of them, and `push` jobs only, because the disk would
  then update less often.
- **Decision:** Docker persistence uses the checked token scope. Rejected:
  an API branch name or a job-supplied ref, because neither proves the full
  ref that may keep disk changes.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Agent](agent.md): own machines and job VMs.
- [Security](security.md): job isolation and job data.
- [Overflow](overflow.md): the jobs that run on Boat.
- [Billing](billing.md#tier-features): the tiers that include Docker disks.
