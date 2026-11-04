# Cache

ciproxy gives each job the cache of GitHub Actions without workflow edits.
This doc states the cache rules for jobs on
[own machines](agent.md#terms) and on [Boat](overflow.md#the-boat-account).

## Terms

- **machine cache:** the cache disk of an own machine.
- **GitHub cache:** the cache service of GitHub Actions that
  `actions/cache` uses.
- **cache scope:** the rule that says which jobs can restore a cache entry.
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
- **Open:** which entries a job on an own machine can restore, so that a
  person can keep a bad entry out of later jobs. An example is an
  entry that a person deleted on GitHub. It waits for the decision on
  entries that leave the GitHub cache.

## Trusted write-back

- Only a job on the default branch writes entries that every branch can
  restore.
- A pull request cannot write an entry that another branch or another pull
  request restores.
- The cache scope of a save comes from GitHub, not from the job, so the
  code of a job cannot choose it.
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
- **Open:** when the job page also shows how many entries the job restored
  and their space, for example for a job on Boat. It waits for the
  decision on the cache counts of Boat jobs.
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
- **Open:** which tiers include Docker disks. It waits for item 8, the tier
  features at launch.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Agent](agent.md): own machines and job VMs.
- [Security](security.md): job isolation and job data.
- [Overflow](overflow.md): the jobs that run on Boat.
- [Billing](billing.md): the tiers.
