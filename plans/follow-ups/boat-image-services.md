# Boat cannot start image services

- **Severity:** High. **Likelihood:** Common. **Effort:** Large.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-17, from the final review of 17 November 2026 of the
  branch `calummoore/ci-proxy-load-balancer`, finding 101, with a new
  problem that its check found: no code creates the Docker bridge
  `ciproxy0`.
- **Source:** Review.
- **Category:** Product bug.
- **Location:** `image/boat-run.sh:9`, `image/boat-run.sh:20`,
  `image/boat-run.sh:26`, `image/boat-run.sh:28`, `image/boat.md:123`,
  `docs/protocol/agent-images.md:24`, and
  `crates/ciproxy-scaler/src/size_policy.rs:28`.
- **Reason:** No option is clearly best: B needs a live Boat proof, C
  raises the Boat cost, and D reverses part of a user decision.

- **Terms:**
  - **Boat:** the overflow provider. ciproxy starts Boat sandboxes, small
    VMs in the customer's own Boat account, while the customer's machines
    are busy.
  - **systemd:** the Linux service manager. It starts the installed
    services of a booted system. `systemctl` is its command.
  - **Image root:** the full userspace of the job image. A Boat job uses a
    copy at `/opt/ciproxy/rootfs` in the shared Boat snapshot.
  - **Wrapper:** `image/boat-run.sh`. It starts the guest of one Boat job
    inside the sandbox.
  - **Guest:** `ciproxy-guest`, the program in the image that runs the
    GitHub runner of one job.
- **Background:**
  - On a customer machine, each job runs in a Firecracker VM that boots
    the image with its own kernel. systemd of the image then starts the
    installed services, such as Docker. `image/provision.sh:27` enables
    them.
  - Boat cannot boot the image. The Boat API has no import of a raw disk
    image, as `image/boat.md:9` says. So a Boat job runs the image root on
    the kernel of the sandbox. `image/README.md:8` says that Boat keeps
    its provider kernel and services.
  - The wrapper starts each Boat job:
    - It stops Boat's own Docker and containerd, at `image/boat-run.sh:8`.
    - It runs itself again in private mount and PID namespaces, at `:9-10`.
      So its Bash is process 1 of the new PID namespace.
    - It mounts an empty `tmpfs` over `/run` of the image root, at `:20`.
    - It starts the image's Docker daemon with `chroot`, at `:26-28`. Then
      it waits up to 60 seconds for `docker info`, at `:31-42`.
    - It starts the guest in its Boat mode with `chroot`, at `:45-46`.
  - `docs/protocol/agent-images.md:5-8` requires the software of the
    GitHub runner image of `ubuntu-latest`, so that a workflow changes only
    its `runs-on` line.
  - A user decision at `docs/protocol/agent-images.md:24-26` says: "one
    image definition serves the job VMs and the Boat sandboxes. Rejected: a
    separate image for each, because a job must behave the same wherever it
    runs."
  - Two public pages give the same promise:
    - `ts/site/src/content/docs/docs/how-ciproxy-works.md:106-108` says
      that every job starts from one image, on your machines and on Boat,
      so only the `runs-on` line of a workflow changes.
    - `ts/site/src/content/docs/docs/security/job-isolation.md:25-26`
      says: "A job behaves the same on your machines and on Boat."
  - The Boat smoke workflow `image/smoke-workflow.yml` checks tool
    versions, `docker version`, and `docker run --rm hello-world`. It
    starts no installed service.
  - `image/check-boat-run.py` tests the wrapper with a stub in place of
    `dockerd`. On 17 November 2026, no live Boat job had passed with the
    wrapper.
  - The live run of 17 November 2026 stopped before the wrapper ran. The
    pinned Boat client rejected Boat's upload acknowledgement, so no Boat
    runner started. So that run did not test the bridge defect below.
  - Boat documents nested virtualization: every sandbox has `/dev/kvm`, so
    Firecracker runs at full speed inside it. Source:
    [Boat virtualization](https://docs.boat.dev/machines/virtualization.md).
  - Boat documents the size `large` with 8 vCPU, 16 GB of memory, and
    111 GB of disk, for $0.072 an hour. The next size, `xlarge`, has
    16 vCPU and 32 GB for $0.20 an hour. It needs the Boat plan of $100 or
    more.
  - The label `ciproxy-4vcpu` has 4 vCPU and 16 GB of memory, at
    `docs/protocol/labels.md:36`.
- **Problem:**
  - No service manager runs for a Boat job. Process 1 is the wrapper's
    Bash, outside the `chroot`.
  - So the chroot check of systemd applies. The commands
    `sudo systemctl start postgresql` and `systemctl restart` print
    "Running in chroot, ignoring command" and exit 0. A step that starts
    PostgreSQL passes, and a later step cannot connect.
  - `sudo systemctl restart docker` also does nothing. Tools that need the
    system bus, such as `timedatectl`, cannot work.
  - The wrapper starts Docker with `--bridge=ciproxy0`, at
    `image/boat-run.sh:28`. No code creates that bridge. Docker does not
    create a bridge with a name other than its default. Its `setupDevice`
    says: "bridge device with non default name ... must be created
    manually".
  - So on a real Boat sandbox, the daemon stops at once. The 60-second
    check fails, and no runner starts. The stub in
    `image/check-boat-run.py` hides this.
- **Impact if no action:** Workflows that start installed services work on
  customer machines and fail when they overflow to Boat. While the bridge
  defect stays, no Boat job starts at all.
- **Root question:** How can a Boat job run on the same booted system as a
  job on a customer machine, so that unchanged workflows behave the same?
- **Options:**
  - **A.** Connect the service commands of the image to the service manager
    of the provider. Configure the image services to use the image root.
    This needs a mapping of services and file systems. Effort: Large.
  - **B.** Start the image's systemd inside the private PID namespace, and
    start the Boat guest through it.
    - Keep the provider's kernel and network.
    - Disable two kinds of image units. The stop command of the vsock guest
      unit reboots the system. The network units can take over the shared
      network.
    - Extend the Boat smoke: start an installed service, and check that it
      is ready.
    - B removes the separate Docker start, and with it the bridge defect.
    - Cost: B needs a proof that the image's systemd runs in a Boat
      sandbox, with Docker under it in that namespace. The provider kernel
      stays, so the kernel, container-detection, and size differences stay.
    - Effort: Large.
  - **C.** Boot the image as a Firecracker VM inside the Boat sandbox.
    - Put the ext4 disk of the release, its kernel, and the pinned
      Firecracker in the shared snapshot.
    - Replace the `chroot` wrapper with a VM launcher. The launcher grows
      the root disk and gives the VM a NAT network. It gives the
      registration to the VM on a private read-only drive.
    - Start the guest in its Boat mode inside the VM.
    - Add the host memory to the Boat size choice, in `select_boat_size`
      at `crates/ciproxy-scaler/src/size_policy.rs:28`.
    - Remove the restore of file capabilities at `image/boat-run.sh:13`,
      because the ext4 disk keeps the file attributes.
    - Extend the Boat smoke with an installed service.
    - The job then gets the kernel and the services of a machine. It also
      gets the vCPU and the memory of its label, as on a machine.
    - C removes the separate Docker start, and with it the bridge defect.
    - Cost: each Boat job pays the overhead of a nested VM. The sandbox
      needs memory for the host and for the VM. So a label whose memory
      fills a Boat size can need the next size. An example is
      `ciproxy-4vcpu`, with 16 GB, on `large`, with 16 GB. Measure both
      costs on `large` first.
    - Effort: Large.
  - **D.** Accept the difference.
    - Change the decision in `docs/protocol/agent-images.md`: a Boat job
      runs without the service manager of the image.
    - Correct the promises at
      `ts/site/src/content/docs/docs/how-ciproxy-works.md:106` and
      `ts/site/src/content/docs/docs/security/job-isolation.md:25`.
    - Explain the difference in `image/boat.md`.
    - Name the workarounds: a `services:` container, or a label that does
      not allow Boat.
    - D keeps the separate Docker start, so the bridge defect needs its own
      fix in the wrapper.
    - Cost: workflows that start services still fail on Boat. D reverses
      part of a user decision and part of a public promise.
    - Effort: Small.
- **Left out:**
  - A list of daemons that the wrapper starts by hand: it leaves the other
    services unsupported.
  - Workflow edits as the fix: they fail the goal of unchanged workflows.
  - An emulated `systemctl` in the `chroot`: it is a second copy of
    systemd. Each unit feature that it does not copy breaks the
    requirement that a job behaves the same. Ubuntu's `postgresql.service`
    even starts its cluster only through a systemd generator.
  - The image's systemd as the init of the whole sandbox, through
    `switch-root`: it stops Boat's controller inside the sandbox. The
    launch inspection at `crates/ciproxy-overflow/src/boat/launcher.rs:44`
    needs that controller.
  - A Boat feature, or another provider, that boots custom disks: ciproxy
    does not control the Boat API, and another provider replaces the
    overflow integration. C gives the same boot inside Boat now.
- **Wider fix:** C. It boots the same system as a customer machine. So it
  also prevents the kernel, container-detection, and size differences that
  B keeps.
- **Previous examples:** None found in the history of `origin/main` or in
  the earlier review reports.
- **Recommendation:** B. It covers the whole class of service-manager
  problems, with `systemctl restart docker` and the system-bus tools. It
  adds no nested VM, needs no larger Boat sizes, and removes the separate
  Docker start with its bridge defect. Choose C if the user also wants the
  same kernel, or if Docker cannot run under the image's systemd in the
  Boat namespace. C best protects the codebase, because it boots the same
  system as a machine.
- **Action:** ask the user, because no option is clearly best. B needs a
  proof that the image's systemd runs inside a Boat sandbox. C changes the
  Boat sizes and the cost. D reverses part of a user decision.
