# Isolation

This page states how ciproxy keeps workspaces and jobs apart, also on a
Mac. [Security](security.md) states sign-in, access, GitHub permissions,
and keys.

## Tenant isolation

- Each workspace has its own machines, [labels](labels.md),
  settings, keys, caches, and data.
- An own machine runs the jobs of every organization of its workspace, each
  in a fresh [job VM](agent.md#job-vms).
- An own machine never runs a job of another workspace.
- The repositories of all the organizations of a workspace keep their cache
  entries and their Docker disks apart, as [Job isolation](#job-isolation)
  says.
- **Decision:** an own machine runs the jobs of every organization of its
  workspace, and tenant isolation holds between workspaces. Rejected: own
  machines for each organization. Reason: one fleet serves the customer,
  and the cache scopes already keep the repositories apart.

## Job isolation

- Each job on an own machine runs in a fresh
  [job VM](agent.md#job-vms).
- So a job never reads the files or the secrets of another job.
- On Boat, each job runs in its own sandbox, as
  [Overflow runners](overflow.md#overflow-runners) says.
- A job VM cannot reach another job VM, the agent, or the host of its own
  machine.
- A job VM can reach the [machine cache](cache.md#cache-tiers) of its own
  machine.
- Apart from the machine cache, a job VM reaches the internet only.
- A job VM cannot reach a private network or a cloud metadata service, also
  when its machine can reach them.
- **Decision:** a job VM reaches the internet only, as on GitHub-hosted
  runners, so a job behaves the same on an own machine and on Boat.
  Rejected: the private networks too, as on a classic self-hosted runner,
  because the code of a pull request could then probe the internal network
  and read the cloud credentials of the host.
- Cache entries and Docker disks follow the isolation rules of
  [Cache scopes](cache.md#cache-scopes),
  [Trusted write-back](cache.md#trusted-write-back), and
  [Docker disks](cache.md#docker-disks).

### Job isolation on a Mac

- The rules of [Job isolation](#job-isolation) hold on a Mac too.
- A macOS job VM uses the NAT network of Virtualization.framework. Its
  traffic passes the host bridge of that network, such as `bridge100`.
- The agent loads pf rules for that bridge. They block each packet from a
  job VM to these addresses:
  - each address of the host of the Mac
  - the private ranges, link-local, `100.64.0.0/10`, and the other reserved
    ranges that a Linux job VM cannot reach
  - each IPv6 address
- The rules allow the DNS and the DHCP of the NAT network, and the
  [machine cache](cache.md#cache-tiers) of the Mac.
- So a macOS job VM reaches the internet, DNS, and its machine cache only.
- The rules also keep the two job VMs of a Mac apart.
- The spike proved the rules for the host, the private ranges, link-local,
  `100.64.0.0/10`, and IPv6. It did not test traffic between two job VMs,
  so the agent work must prove it.
- **Decision:** the pf rules block all IPv6 from the job VMs. Reason: in
  the spike, a job VM reached the host over IPv6 link-local without this
  rule. Rejected: IPv6 rules that allow only the internet, because the
  spike proved only the block of all IPv6, and each IPv6 rule is one more
  way out.
- **Decision:** a Mac isolates its job VMs with pf on the bridge of the NAT
  network of Virtualization.framework. Rejected: a network stack of our
  own in the agent, because the NAT network with pf passed the spike.
- The macOS image turns off System Integrity Protection (SIP) in the guest,
  as the GitHub macOS runner images do. The upstream build of the image
  writes the system TCC database, and SIP blocks that write.
- So a job can change the system files of its job VM. This is safe,
  because each job VM is fresh and the agent never uses its disk again, and
  because the pf rules of the host, outside the guest, isolate it.
- **Decision:** the macOS image keeps SIP off in the guest. Rejected: SIP
  on, because the upstream scripts of the GitHub runner images could then
  not set up the image. Reason: the isolation comes from the host and from
  the fresh VM, not from the guest.
- The agent removes its pf rules when it is uninstalled, and it keeps every
  other pf rule of the Mac.

## Related docs

- [Security](security.md): sign-in, roles, GitHub App permissions,
  repository access, and key custody.
- [Data and retention](security-data.md): secrets, job data, and how long
  ciproxy keeps each kind of data.
- [Workspaces](workspaces.md): the organizations of a workspace, and what
  the workspace owns.
- [Agent](agent.md): job VMs, the agent, and removal.
- [Agent images](agent-images.md): the macOS image and its SIP setting.
- [Cache](cache.md): cache scopes, trusted write-back, and Docker disks.
- [Overflow](overflow.md): the Boat account and its sandboxes.
- [Labels](labels.md) and [Job routing](job-routing.md): labels and
  runners.
