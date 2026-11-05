# The area leaves no room for images

- **Severity:** High. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** Yes.
- **Kind:** Needs decision.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, capacity finding 12.
- **Source:** Implementation.
- **Category:** Product bug.
- **Location:** `crates/ciproxy-agent/src/area.rs`,
  `crates/ciproxy-agent/src/image_files.rs`,
  `crates/ciproxy-agent/src/image_registry.rs`, and
  `docs/protocol/job-routing.md`, "Disk".
- **Reason:** The correction changes the approved file size and limits
  image writes. The user must choose before either change.

- **Terms:**
  - **Area:** The sparse XFS file that holds images, VM disks, Docker
    disks, and the machine cache.
  - **Base:** The retained root disk from which each fresh VM reflinks.
  - **Image share:** An enforced budget for retained images and image
    downloads, taken from the host reserve.
  - **Update peak:** The old base, pending archive, and new base coexist
    during an image update.
  - **Project quota:** An XFS limit on the blocks mapped by files in a
    configured tree.
- **Background:** The approved host preparation uses one XFS file on
  every filesystem. The orchestrator specified a file of three quarters
  of the host capacity. The protocol gives those same three quarters to
  VM reservations and the machine cache. It keeps the remaining quarter
  for the system and images. Reflink bases and their VM copies must share
  a filesystem.
- **Problem:** XFS metadata and retained bases also occupy the area.
  Admission can therefore reserve more VM disk than the area can back.
  Image updates add the archive and another base. Merely enlarging the
  file still lets unlimited image writes consume reserved VM space.
- **Impact if no action:** A job can receive write errors before its own
  disks are full. Concurrent image preparation can exhaust space needed
  by a running job.
- **Evidence:**
  - A disposable ext4 volume reported 1,020,702,720 bytes. Its XFS file
    and VM file each had 765,527,040 logical bytes. A retained nonzero
    base occupied 134,217,728 bytes. Synchronous VM writes failed with
    `ENOSPC` after 525,336,576 bytes. Distinct source and clone inodes
    initially shared physical extents. The base stayed unchanged.
  - A separate XFS probe enforced a 64 MiB image project limit. Further
    image writes stopped at that limit while 238,006,272 bytes remained
    free. A cross-project VM reflink then accepted 128 MiB of private
    writes. Both probes removed their mounts and temporary files. They
    used no KVM and did not install the agent.
- **Root question:** Where do images and filesystem overhead get their
  space, so the job disk stays whole for VMs and the cache?
- **Options:**
  - **C.** Keep the file and admission rules. Change the protocol to allow
    images and metadata to consume job disk and permit earlier write
    failures. This weakens the disk promise. Effort: Small.
  - **A.** Enlarge the file for the full job disk, an image share, and
    measured overhead. Enforce the share in every image write path.
    Count retained, held, and pending images together. This needs runtime
    accounting and cleanup recovery. Effort: Medium.
  - **F.** Use the larger file and shared sizing ledger of A. Enforce the
    image share with an XFS project quota. Keep VM copies outside that
    image project. The kernel checks every image write. This needs XFS
    quota support, and an oversized update keeps the old image.
    Effort: Medium.
  - **B.** Keep the file at three quarters. Subtract images and metadata
    from the job capacity used by admission and the scaler. This changes
    the protocol rule and the minimum disk requirement. An exact 80 GB
    capacity no longer fits the default 60 GB job. Effort: Large.
  - **E.** Keep bases outside the area as shared read-only drives. Put
    writable device-mapper snapshots in each guest. This replaces the
    reflink-root design and needs guest, image, and KVM work. It still
    needs an enforced image budget on the backing filesystem.
    Effort: Large.
- **Left out:** A whole-host-size file still needs enforced budgets and
  loses the smaller file's system-space backstop. Native reflinks alone
  do not serve ext4. The user rejected full-copy fallback. A base outside
  the area cannot reflink into it. A quota per VM adds nothing to its
  fixed file sizes. A free-space check before start cannot protect later
  writes. Letting an unbounded fetch fail on a full area can fail a VM
  first. Deleting the current base before extraction blocks new starts.
- **Wider fix:** F. Kernel enforcement covers every writer in the image
  tree, including future paths that omit a userspace check.
- **Previous examples:** No earlier repository case found. The scratch
  project-quota probe confirmed cross-project reflinks and independent VM
  writes on one kernel. The
  [XFS quota manual](https://man7.org/linux/man-pages/man8/xfs_quota.8.html)
  describes directory-tree enforcement.
- **Related:**
  - [Storage measurement](own-machine-storage-measurement.md) supplies the
    actual update peak and overhead needed to size the share.
  - [Cache integration](agent-cache-area-integration.md) must honor its
    own budget before a VM starts. An image quota does not reclaim cache.
- **Recommendation:** F, changed from A after the options check and live
  filesystem probe. Kernel enforcement protects every image write with
  less runtime accounting.
- **Action:** ask the user, because F enlarges the explicitly
  three-quarter file, adds an image-write limit, and requires XFS quotas.
- **Sizing constraint:** Job disk stays at three quarters. System space,
  the image share, and all filesystem and VM support-file overhead must
  fit together inside the remaining quarter. Metadata must not be added
  outside that total. For an illustrative 80 GB capacity, 60 GB job disk,
  8 GB system budget, and 0.5 GB overhead leave an 11.5 GB image share and
  a 72 GB backing file. These are illustrative values, not measured image
  requirements or approved defaults.
- **Current outcome:** Neither larger-file option nor a quota is applied.
  The code keeps the approved three-quarter size pending the answer.
