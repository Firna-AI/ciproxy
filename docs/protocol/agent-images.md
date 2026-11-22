# Agent images

This doc states the requirements for the job images: the Linux image with
its software, its two builds, and its releases, and the macOS image.
[Job VMs](agent.md#job-vms) on Linux boot the machine image. Boat
[sandboxes](overflow.md#terms) start from the Boat snapshot. Job VMs on a
Mac boot the macOS image.

## Terms

- **image definition:** the pinned recipe of the open-source GitHub runner
  images, its version pins, and the parts that ciproxy adds, such as the
  runner and the guest.
- **guest:** the ciproxy program in the image that starts the runner of one
  job and reports the end of that runner.
- **machine image:** the build of the image definition that job VMs boot.
  It holds its own kernel.
- **Boat snapshot:** the build of the image definition on Boat's own
  Ubuntu system. Each sandbox starts from it.
- **Boat's controller:** the service of Boat in each sandbox that runs the
  commands and the file transfers of the Boat API.
- **macOS image:** the job image of macOS VMs. It holds macOS, Xcode, the
  software of the GitHub runner image that `macos-latest` names, the
  runner, and the guest.
- Each image has one [platform](agent.md#platforms). The machine image and
  the Boat snapshot are Linux x64. The macOS image is macOS arm64.

## Software

- This section and the next three sections state the Linux image.
- The image builds on the open-source GitHub runner images.
- The image has the software of the GitHub runner image that
  `ubuntu-latest` names, so a workflow keeps its YAML apart from the
  `runs-on` line.
- The runner in the image is always a version that GitHub still accepts.
- So ciproxy releases a new image before GitHub stops accepting the runner
  of the current image.
- A job can run its steps in a Docker image of its own choice, with the
  `container:` key of GitHub Actions, on every tier.
- **Decision:** every label of a platform starts from the one image of
  that platform until launch, and a
  job picks other software with a job container. Rejected for now: a
  catalog of images that each label names, custom VM images, and toolset
  layers. Reason: the user chose this on 5 November 2026, and plans the
  catalog and custom images after launch.
- No workspace can run its jobs on a custom VM image yet.
- Custom VM images come later, on Business, as
  [Tier features](billing.md#tier-features) says.

## One definition, two builds

- **Decision:** one image definition serves the job VMs and the Boat
  sandboxes. Each release builds it as the machine image and as the Boat
  snapshot. Rejected: a separate image for each, because a job must behave
  the same wherever it runs.
- Both builds run the same recipe with the same version pins.
- The recipe selects some software at its latest version. The image
  definition pins each of these versions, and both builds use the pins.
- Versions that no pin covers can differ between the builds. Examples are
  package revisions and the packages of each base system.
- Each build reports the software versions that it installed.
- Each release records each version difference between the two reports.
- A version difference does not stop a release.
- A job starts with Docker and containerd running, in both builds.
- The services that the recipe disables, such as the databases and the web
  servers, stay stopped until a step starts them, in both builds.
- A step can start, stop, and restart an installed service with
  `systemctl`, in both builds.

## The machine image

- The image root is built at the smallest size that holds the full image.
- Each job root disk reserves 1% of its blocks for root.
- The free disk of the label remains available to the unprivileged runner,
  as `df` reports it at job start.
- Each image release supplies its logical root disk bytes.
- The agent keeps those facts with the verified image and its preparation hold.
- It calculates the root allocation from the held release and the free
  label disk.
- A job VM starts from the newest image that its machine has, and a
  running job keeps its image.
- Each runner retains its original free-size request and actual allocation.
- Publication and restart do not change those facts for an existing runner.

## The Boat snapshot

- **Decision:** the Boat build installs the image definition on a sandbox
  from Boat's base image, and Boat saves that sandbox as the snapshot.
  Rejected: the files of the machine image in a `chroot`, because no
  service manager runs their services there. Rejected: the systemd of the
  image in a private namespace, because no test showed that it works on
  Boat. Rejected: a nested VM in each sandbox, because it needs larger Boat
  sizes.
- The snapshot holds the full software of the recipe and the parts that
  ciproxy adds.
- A Boat job finds the pinned version of each tool of the image definition
  first, as on a job VM.
- **Decision:** the Boat build removes Boat's own commands and apt packages
  for each such tool, and the settings that put them first, before the
  install steps. Rejected: keeping Boat's tools first, because a job would
  then use other versions than on a job VM. Rejected: removing Boat's other
  tools too, because they change no pinned version.
- The snapshot leaves out only the parts that only a job VM needs, and the
  setup that Boat owns. Examples are the kernel, the boot and disk setup,
  the network configuration, and the reboot that ends a job VM.
- The Boat build records the result of each install step of the recipe.
- It also records each step that it skips and each upstream script or test
  that it changes, with the reason.
- These differences from a job VM stay:
  - A Boat job runs on Boat's kernel.
  - It runs on the Ubuntu point release and the base packages of Boat's
    base system.
  - Boat's controller, its user accounts, its desktop services, its network,
    its resolver, and its firewall stay in the sandbox.
  - Boat's extra tools that the image definition does not install stay.
    The release reports them.
  - The `docker` group keeps its group ID from Boat's base system.
  - `/usr/local` and `/usr/local/bin` keep their owner from Boat's base
    system.
  - [Overflow runners](overflow.md#overflow-runners) gives the size and the
    cache of a Boat job.
  - A Boat job gets no [Docker disk](docker-disks.md).
- **Decision:** the snapshot keeps Boat's kernel, base system, and provider
  services. Rejected: the kernel and the services of the machine image in
  their place, because Boat starts only its own kernel, and each launch
  needs Boat's controller.
- Boat's controller keeps working during the build and in each sandbox.
- In each restored sandbox, a command that Boat's controller runs has the
  UTF-8 locale that Boat's base system sets. This holds from the first
  command after the restore.
- Boat's controller adds no locale warning to the stderr of such a command.
- Boat's own users and the runner account can use Docker in each sandbox.
- A Boat job runs on the system of its sandbox, under its service manager,
  with no `chroot` and no private namespace.
- A restored sandbox keeps the owner, the mode, the file capabilities, and
  the ACLs of each file of the snapshot.
- The snapshot holds no runner registration, token, or key.
- The snapshot keeps no file or setting that only the build uses, such as
  its build folder and its build apt settings.
- Each launch gives the guest its registration in a file that only root can
  read.
- The runner takes no job before Docker and containerd run the versions of
  the snapshot.
- **Decision:** each launch brings the services of the restored sandbox up
  to date, and then starts the guest as a service. Rejected: the services
  that the snapshot enables, because a Boat restore keeps the processes of
  Boat's base system and does not boot the saved system.

## The macOS image

- The macOS image has the software of the GitHub runner image that
  `macos-latest` names, on arm64, with Xcode.
- So a macOS workflow keeps its YAML apart from the `runs-on` line.
- A macOS job VM has no Docker, as on the GitHub macOS runners.
- ciproxy builds the macOS image automatically. The build installs macOS
  from Apple's restore image into a VM, completes the Setup Assistant over
  VNC, and runs the macOS scripts of the open-source GitHub runner images.
- The build installs Apple's newest full restore image, then updates the
  guest to the pinned macOS version: macOS 26.7.3 (25G342).
- The pinned GitHub runner image runs macOS 26.7.2. Software Update no
  longer offers 26.7.2, so the guest is one security update newer.
- The build turns off System Integrity Protection (SIP) in the guest, as
  the GitHub macOS runner images do, because the upstream scripts write
  the system TCC database.
  [Job isolation on a Mac](security.md#job-isolation-on-a-mac) says why
  this is safe.
- Then the build adds the runner and the guest, as for the machine image.
- ciproxy serves each release of the macOS image as it serves the machine
  image. The agent downloads and verifies it in the same way.
- **Decision:** ciproxy builds and serves a ready macOS image with Xcode.
  Rejected: an image that each Mac builds itself, because the first job of
  a new Mac would wait hours. Rejected: a customer image, because custom
  images come later. Reason: the user has approval to distribute the image
  with Xcode, and chose this on 21 November 2026.
- Unlike a download of the machine image, the agent continues a broken
  download of the macOS image from the bytes that it has. It keeps these
  bytes across download commands and agent restarts while the release ID,
  the SHA-256, and the byte size stay the same.
- The agent checks the byte count and the SHA-256 of the whole file at the
  end. A failed check deletes the bytes, and the next download starts from
  zero.
- An HTTP error status, such as the answer to an expired URL, fails the
  download command and keeps the bytes. The next download command brings a
  fresh URL.
- **Decision:** the agent continues with HTTP range requests, and the
  download URL keeps the 2-hour expiry of the machine image URL. Rejected:
  a URL valid for 12 hours, because a leaked URL to the image with Xcode
  then stays valid longer. Rejected: an image in parts with a SHA-256 for
  each part, because it changes the image form and the publish step.
  Reason: the user chose this on 23 November 2026.
- The macOS image holds the VM disk and the platform files that
  Virtualization.framework needs to start it.
- The macOS image has no Boat snapshot, because a macOS job never runs on
  Boat, as [Overflow runners](overflow.md#overflow-runners) says.
- The image disk has a fixed logical size, now planned at 200 GB. It holds
  free space for the jobs inside its APFS container.
- Each release supplies the logical bytes of its disk, as the machine image
  does.
- Each release also supplies the free bytes that the build measured in the
  guest. This is the free disk of each macOS job VM.
- The agent makes an APFS clone of the image disk for each job VM, and does
  not grow it.
- **Decision:** the macOS image disk is fixed, and the agent does not grow
  it. Rejected: a small image disk that the agent grows for each label,
  because the build must then delete the recovery container of the guest,
  the guest must resize its container at each boot, and a saved VM state
  then serves only one disk size. This is a proposal until the user
  decides it.
- A job VM starts from the newest macOS image that its Mac has, and a
  running job keeps its image.
- The runner in the macOS image is always a version that GitHub still
  accepts, as for the machine image.
