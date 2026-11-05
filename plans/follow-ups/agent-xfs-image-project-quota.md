# Kernel enforcement of the image budget

- **Severity:** Medium. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, capacity finding 12, option F.
- **Source:** Design.
- **Category:** Code structure.
- **Location:** `crates/ciproxy-agent/src/area_mounts.rs`,
  `crates/ciproxy-agent/src/area.rs`,
  `crates/ciproxy-agent/src/image_files.rs`,
  `crates/ciproxy-agent/src/image_registry.rs`,
  `crates/ciproxy-agent/src/host_paths.rs`,
  `crates/ciproxy-agent/src/image_space.rs`,
  `crates/ciproxy-agent/src/vm_files.rs`,
  `crates/ciproxy-agent/image-budget.md`, and
  `docs/protocol/agent.md`, "Install".
- **Reason:** F and G add a host requirement that only the user can
  approve. Option A is adequate without either quota.

- **Terms:**
  - **Area:** The sparse file, formatted as XFS and mounted at
    `/var/lib/ciproxy/area`, that holds the images, VM disks, Docker disks,
    and machine cache.
  - **Base:** The kept root disk of an image, from which each fresh VM disk
    is a reflink copy.
  - **Job disk:** Three quarters of the disk left after the image budget,
    which admission gives to the VMs and the machine cache.
  - **Image budget:** The separate space reserved first for images and the
    kernel, configuration, executable, and support files of each VM jail.
  - **Project quota:** An XFS limit on files with one inherited project
    number, except the unlimited default project 0.
- **Background:**
  - Install prepares the area on any filesystem, so that each VM disk can be
    a reflink copy of a base.
  - Reflinks work only inside one filesystem. So the bases must be in the
    area with the VM disks.
  - "Disk" in `docs/protocol/job-routing.md` subtracts the image budget
    first. The host keeps a quarter of the remainder for the system and
    filesystem overhead, and job disk gets three quarters.
  - On 5 November 2026 the orchestrator chose option A as fix now. The area
    holds the job disk, the image budget, and the XFS overhead measured
    after formatting.
  - Current sizing keeps the image budget apart from job disk. Filesystem
    overhead uses the remaining host reserve. A 250 GB minimum machine
    gives jobs 82.5 GB with the default budget. The default job needs 80 GB.
  - Under A, the agent reserves budget before each physical image or VM
    support write. It measures allocation under `images/` and the VM jail
    tree. It excludes the root and Docker payloads already in the job disk.
  - It reclaims unused older bases before admission. The
    `crates/ciproxy-agent/image-budget.md` guide describes the accounting.
    No quota is required.
  - The image budget defaults to 140 GB. It holds two full 58.3 GB images
    and one 18.3 GB download, plus support headroom. Host measurement must
    still confirm the complete image and support peak at full VM occupancy.
  - A needs no answer. It is adequate for the own-machine agent, and it does
    not wait for this item.
  - This item is an optional design choice. It asks whether the kernel
    should also enforce the budget, at the cost of a new host requirement.
- **Problem:** Under A, the image budget depends on the agent's accounting
  in each image or VM support write path. The kernel gives no second check.
  No finding shows a path in A that skips or miscounts that accounting.
  Only a future path that did so could let image writes take job disk.
- **Impact if no action:** A stays the only enforcement. Review and tests
  must keep each new image or VM support write inside the accounting.
  A missed path could give a job write errors before its own disks are full,
  as the old three-quarter area did.
- **Evidence:**
  - A disposable ext4 volume reported 1,020,702,720 bytes. The old area
    file and a VM file each had 765,527,040 logical bytes, three quarters of
    that volume. A retained nonzero base occupied 134,217,728 bytes.
  - Before the VM writes, the area had 525,484,032 bytes free. So XFS kept
    105,825,280 bytes of the file for its own use.
  - Synchronous VM writes failed with `ENOSPC` after 525,336,576 bytes.
    Distinct source and clone inodes initially shared physical extents. The
    base stayed unchanged.
  - A separate probe mounted a fresh XFS file with project quotas. It gave
    the images directory project 42 with a 64 MiB hard limit.
  - A 32 MiB base in project 42 reflinked into a jobs directory in project
    0. The two files shared their physical extents.
  - A further image write stopped at the 64 MiB limit with `ENOSPC`, while
    238,006,272 bytes remained free. The VM file then accepted 128 MiB of
    private writes.
  - Both probes removed their mounts and temporary files. They used no KVM,
    did not install the agent, and ran on one kernel only.
  - `xfs_quota` ships in `xfsprogs`, which install already prepares.
- **Root question:** What protects the job disk if a future image or VM
  support write misses the agent's accounting?
- **Options:**
  - **A.** Keep A alone. Each new image or VM support write path uses the
    agent's accounting. Review and tests check that it does. It adds no
    host requirement. Effort: Small.
  - **F.** Keep the area size and the image budget of A. Mount the area
    with project quotas, and give the images directory a project whose hard
    limit is the budget. Keep VM disk copies outside that project. Its
    coverage is only `images/`. VM support files still rely on A alone.
    The kernel stops image-tree writes at that limit, but the combined
    image and support budget still needs the agent's accounting. This needs
    XFS quota support in every machine's kernel, a new host requirement in
    "Install" of
    `docs/protocol/agent.md`. An update that needs more than the budget
    fails and keeps the old image. Effort: Medium.
  - **G.** Put images and the VM jail tree under one inherited project with
    the budget as its limit. Give each root and Docker payload project 0
    before it receives data. New support files then count by default.
    The VM start path must establish those exceptions before reflinking.
    A scratch probe must confirm that a project-0 destination inside an
    inheriting folder can reflink from its base. This needs the same XFS
    quota support as F. Effort: Medium.
- **Left out:**
  - **C.** Keep the three-quarter file, and let the images and the XFS
    metadata use the job disk. It weakens the job-disk promise of "Disk".
  - **B.** Keep the three-quarter file, and subtract the images and the
    metadata from the job disk of admission and the scaler. It changes the
    job capacity and the minimum machine. At the time this option was
    rejected, an exact 80 GB machine would no longer fit the default 60 GB
    job. Current sizing reserves the 140 GB image budget before the split.
  - **E.** Keep the bases outside the area as shared read-only drives, with
    a writable device-mapper snapshot in each guest. It replaces the
    approved reflink roots, and it needs guest, image, and KVM work. It
    still needs an enforced image budget on the host filesystem.
  - A quota for each VM: it adds nothing to the fixed sizes of the VM
    files.
  - A free-space check before each VM start: it cannot protect writes after
    the start.
  - A fetch with no limit that fails on a full area: the first writer to
    hit the full area can be a VM.
  - An area as large as the host filesystem: it still needs an enforced
    budget, and it loses the backstop that the smaller file gives the
    system space.
  - The native reflinks of the host filesystem: ext4 has none.
  - Full copies of the base: the user rejected them on 5 November 2026.
  - A base outside the area: it cannot reflink into the area.
  - Deleting the current base before an extraction: no VM could start
    during an update.
  - Preallocating each VM disk: it cannot reserve private copy-on-write
    blocks of the shared root without the rejected full-copy approach.
  - A user or group quota: it needs the same kernel support and cannot
    follow support files assigned to different jail users.
  - A factory for budgeted writers alone: it cannot control the jailer or
    another process that builds a path directly.
- **Wider fix:** G, changed from F after the options check. Its default
  includes every image and VM support writer. F protects only the image
  tree, and A must guard its other writers.
- **Previous examples:** The first host preparation sized the area at three
  quarters with no image limit, and the first probe above shows the result.
  The orchestrator chose A to fix that case. No other repository case
  found. The
  [XFS quota manual](https://man7.org/linux/man-pages/man8/xfs_quota.8.html)
  describes directory-tree enforcement.
- **Related:**
  - [Storage measurement](own-machine-storage-measurement.md) measures the
    space of an image update, with the old base, the archive, and the new
    base together. That value sets the real image budget. Under F, it also
    sets the quota limit.
  - [Cache integration](agent-cache-area-integration.md) keeps its own
    budget before a VM starts. An image quota does not reclaim cache space.
  - `crates/ciproxy-agent/scripts/storage-regression.py` runs production
    sizing and bounded writes on disposable ext4/XFS. Its opt-in target is
    `crates/ciproxy-agent/tests/storage_regression.rs`. Extend that proof
    for the selected quota and cross-project placement before enabling it.
  - [Interrupted uninstall](agent-interrupted-install-uninstall.md) fixes
    the removal of a partly prepared area. A quota step in preparation must
    keep that removal route. Neither item waits for the other.
- **Recommendation:** G, changed from F after the options check. Its
  default covers both image files and VM support files. F remains a
  smaller change. A stays adequate and does not wait for either option.
- **Action:** ask the user, because F and G add XFS quota support as a host
  requirement. Only the user can approve that requirement.
- **Current outcome:** The agent work applies A, the orchestrator's fix of
  5 November 2026. A needs no answer. Neither F nor G is applied. Install
  sets no project quota and adds no host requirement.
