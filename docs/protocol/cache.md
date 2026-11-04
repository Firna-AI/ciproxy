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
- A job restores the entries of its own branch and of the default branch.
- A job of a pull request also restores the entries of its base branch.
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

- Only a job on the default branch writes entries that every branch can
  restore.
- A pull request cannot write an entry that another branch or another pull
  request restores.
- A job of a `pull_request_target` event counts as a pull request, also
  when its ref is the default branch. Reason: its job often runs code from
  the pull request.
- The cache scope of a save comes from GitHub, not from the job, so the
  code of a job cannot choose it.
- The control plane reads the ref, the event, and the head repository of
  the run of each job from the GitHub API, as
  [GitHub App permissions](security.md#github-app-permissions) says.
- **Decision:** only jobs on the default branch write entries for every
  branch. Rejected: shared writes from every branch, because a pull request
  could then poison the cache.

## Cache tiers

- On an own machine, the machine cache gives the contents of a restored
  entry when it has them.
- Otherwise the contents come from the GitHub cache, and the machine cache
  keeps them.
- A save goes to the machine cache and through to the GitHub cache, so
  other own machines and jobs on Boat can restore it.
- A job on Boat uses the GitHub cache only.
- The job page shows the cache source of each job: "This machine" or "The
  GitHub cache".
- A job on Boat has the cache source "The GitHub cache".
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
- One job at a time keeps its changes to a Docker disk: the first job on
  the default branch that uses the disk.
- Each other job of the repository works on a copy of the disk, also each
  job that does not run on the default branch.
- ciproxy discards a copy when its job ends.
- The branch of a job comes from GitHub, so the code of a job cannot choose
  it.
- The Docker disk of one repository never attaches to a job of another
  repository.
- A job on Boat gets no Docker disk.
- When a Docker disk cannot attach, the job runs without it.
- The machine page shows how many repositories have a Docker disk on the
  machine, and the space that these disks take.
- Only Team and Business include Docker disks, as
  [Tier features](billing.md#tier-features) says.
- A job of a workspace on Free gets no Docker disk.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Agent](agent.md): own machines and job VMs.
- [Security](security.md): job isolation and job data.
- [Overflow](overflow.md): the jobs that run on Boat.
- [Billing](billing.md#tier-features): the tiers that include Docker disks.
