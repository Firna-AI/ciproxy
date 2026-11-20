# Agent images

This doc states the requirements for the job image: its software, its two
builds, and its releases. [Job VMs](agent.md#job-vms) boot the machine
image. Boat [sandboxes](overflow.md#terms) start from the Boat snapshot.

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

## Software

- The image builds on the open-source GitHub runner images.
- The image has the software of the GitHub runner image that
  `ubuntu-latest` names, so a workflow keeps its YAML apart from the
  `runs-on` line.
- The runner in the image is always a version that GitHub still accepts.
- So ciproxy releases a new image before GitHub stops accepting the runner
  of the current image.
- A job can run its steps in a Docker image of its own choice, with the
  `container:` key of GitHub Actions, on every tier.
- **Decision:** every label starts from the one image until launch, and a
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
