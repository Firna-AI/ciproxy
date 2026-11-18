# Cache handoff

This page states how the runner of a job on an
[own machine](agent.md#terms) hands the cache rights of the job to the
host, and when the steps of the job start. [Cache](cache.md) states the
other cache rules and defines the [cache terms](cache.md#terms).

- Before the first step of a job on an own machine, its runner gives the
  host the original cache URLs, runtime token, and cache rights of the job.
- The host accepts that first handoff only over the connection that
  configured the runner.
- After a reconnect before the first handoff, the job uses only the GitHub
  cache until it ends.
- The host saves only the cache rights of the first handoff and the scheme,
  host, and port of its cache and results URLs. These are the origins.
- The host keeps the runtime token and the full URLs in memory only.
- A later handoff of the job cannot change those origins or the original
  cache rights. It can renew a URL on the same origin.
- The steps of the job start only after the host decides: the machine cache
  is ready with the original handoff, or the job uses only the GitHub cache.
- The runner sets no time limit of its own for that decision, and it does
  not fail the job.
- GitHub's job time limit ends a job whose host never decides.
- **Decision:** the host trusts only a first handoff over the connection
  that configured the runner. Rejected: rules on the destination alone,
  because a job could still claim cache rights that it lacks.
- **Decision:** the steps wait for the host decision, with no time limit.
  Rejected: a fallback after 30 seconds or after a socket error, because job
  code could then send the first handoff itself.
- **Decision:** first runs keep the machine cache, and root in the job VM
  stays unchanged. Rejected: the GitHub cache only for new jobs, or a
  restricted root, because they remove a feature or can break jobs.

## Related docs

- [Cache](cache.md): the cache scopes, the restore order, trusted
  write-back, and the fallback to the GitHub cache.
- [Agent](agent.md#job-vms): the job VMs of own machines.
- [Security](security.md#job-isolation): what a job VM can reach.
