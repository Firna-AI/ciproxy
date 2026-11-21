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
- **GitHub fill:** the copy that the machine cache keeps of an entry that a
  job restored from the GitHub cache.
- **entry facts:** the ID, ref, key, version, and creation time of an entry
  of the GitHub cache, as GitHub reports them.
- **job ID pair:** the two IDs of one job that the control plane binds to
  the one registered runner of the job. One is the ID that the runner
  receives with the job. The other is the ID in the messages of GitHub's
  [scale set](../architecture.md#components) about the job.

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
- "Clear cache" never deletes a [Docker disk](docker-disks.md) of the
  machine.
- **Decision:** the machine cache keeps entries after the GitHub cache
  evicts them, and "Clear cache" removes a bad entry. Rejected: restoring
  only the entries that the GitHub cache still has, because the machine
  cache could then serve no more entries than the GitHub cache holds.
- **Decision:** "Clear cache" acts on one machine. Reason: it sits with the
  cache use of that machine.

## Restore order

- A restore searches the cache scopes in the order that
  [Cache scopes](#cache-scopes) gives, and its keys in the requested order.
- For each key, an exact match comes before a prefix match.
- Among prefix matches, the newest entry wins.
- For a job with a checked ref, this order covers the entries of the
  machine cache and the GitHub cache together.
- So a match in one cache never hides an earlier match in the other cache.
- This order needs the [entry facts](#terms) of the GitHub cache.
- When they are missing or inconsistent, a job with a checked ref gets the
  contents from the GitHub cache, or it sees a cache miss.
- The job never fails for this reason.
- The entry facts come from GitHub, not from the job.
- The control plane reads them with the "Actions" read permission of the
  [GitHub App](security.md#github-app-permissions).
- It gives the entry facts of a repository only to the machine that runs a
  job of that repository.
- Each restore uses the entry facts that GitHub reports at that time.
- A saved entry or a GitHub fill takes its place in the order from the
  entry facts of its GitHub cache entry.
- The machine cache keeps those facts after a restart, and after GitHub
  deletes or evicts that GitHub cache entry.
- Those facts never change which jobs can restore the entry from the
  machine cache.
- The machine never guesses the ref or the creation time of an entry.
- **Decision:** one order covers both caches. Rejected: two lookups with a
  check between them, because two orders can disagree. Rejected: a shared
  entry index across machines, because it adds shared state.
- **Decision:** the entry facts come from GitHub through the control plane.
  Rejected: a ref from the job, because job code can choose it.

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
- It checks the assigned repository, run, and runner.
- The control plane names the job to the machine by the ID that its runner
  received.
- The control plane also gives the machine the [job ID pair](#terms).
- The job ID of the token must equal one ID of that pair.
- No job name or other alias of the job matches.
- **Decision:** the job ID of the token can equal either ID of the pair.
  Rejected: one fixed ID of the pair, because the two IDs of one job can
  differ, and GitHub does not document which ID the token holds. Reason:
  both IDs belong to the one registered runner of the job, and the token
  must also pass every other check.
- The [verified live sample][job-id-proof] had equal Worker, scale-set,
  and signed job IDs. It proves that the claim named the bound job. It
  cannot distinguish the two namespaces when the IDs differ.
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
[job-id-proof]: ../../crates/ciproxy-cache-sidecar/README.md#runtime-token-job-identity

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
- On Machines, cache hits cover restores of own machines with
  [measured](#machine-figures) cache figures. The page names the others.
- With no measured machine, the cache hits say "Unknown", not "No restores yet".

## Machine figures

- The cache figures and the [Docker disk figures](docker-disks.md#terms) of a
  report are each measured or unavailable. A measured zero is a true zero.
- Each producer reports the figures that it did not measure as unavailable.
- So the cache figures are unavailable until the machine cache knows its total
  space and recovers its disk, and while it cannot read its saved counts.
- The agent tries to send the machine cache its total space before each first
  report: at registration, and after a start of the agent or the machine cache.
- The agent logs each failed read. The rest of its report still arrives.
- No reader counts an unavailable figure as zero, or as no cache use of a job.
- Each page shows "Unknown" in place of an unavailable figure.
- Figures without a mark, as from an earlier agent, count as measured.
- **Decision:** each producer marks the figures that it did not measure.
  Rejected: uncertain zeros, because they make a true empty state unclear.
  Rejected: old figures, because they need stored state and an age rule.

## The cache handoff

- [Cache handoff](cache-handoff.md) states how the runner of a job on an own
  machine hands the cache rights of the job to the host, and when the steps
  of the job start.

## Fallback to the GitHub cache

- A failure of the machine cache never fails a job.
- The job then uses the GitHub cache directly, or it sees a cache miss.

## Docker disks

- [Docker disks](docker-disks.md) states the rules for the Docker disks of
  own machines.

## Related docs

- [Architecture](../architecture.md): the components and the data flow.
- [Agent](agent.md): own machines and job VMs.
- [Cache handoff](cache-handoff.md): the handoff of the cache rights of a
  job to the host.
- [Docker disks](docker-disks.md): the Docker disks of own machines.
- [Security](security.md): job isolation and job data.
- [Overflow](overflow.md): the jobs that run on Boat.
