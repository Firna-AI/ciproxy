# Image storage requirements need host measurement

- **Severity:** Medium. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-05, from
  `conductor/ciproxy-mvp-milestone-9-agent`, host storage checks.
- **Location:** `image/build.sh`, `image/README.md`,
  `crates/ciproxy-agent/src/image_extract.rs`,
  `crates/ciproxy-agent/src/image_registry.rs`, and
  `docs/protocol/agent.md`, "Install".
- **Reason:** Measuring the full image needs a KVM build host and changes
  to image tooling outside the agent host-preparation task.

- **Terms:**
  - **Base:** The retained root image copied for each new job VM.
  - **Update peak:** Space occupied by retained or held bases, a pending
    download, and extraction during image publication.
  - **Usable capacity:** Storage that the selected backing filesystem
    reports, after its own geometry and metadata overhead.
- **Background:** The image build already emits a measurement report.
  The agent now preserves zero runs as sparse holes during extraction.
  Image replacement keeps the old base until publication and can also
  hold a base during VM preparation. Host storage sizing needs these
  allocations as well as the guest's root size. A nominal 250 GB device
  can report less usable filesystem capacity than the protocol example.
- **Task:** Extend the existing image and host measurement to record
  logical root size, allocated sparse bytes, archive size, update peak,
  per-VM kernel/runtime support files, and filesystem overhead. Record
  nominal device capacity, selected filesystem capacity, target area
  length, and mounted usable capacity separately. Compare the results
  with the image budget of the area and the default VM's full reservation.
  Measure support use with all intended VM slots occupied, including warm
  VMs. Validate the 140 GB image-budget default for each supported machine
  capacity against the combined image and support peak. The default holds
  two full 58.3 GB images and one 18.3 GB download, plus support headroom.
- **Impact if no action:** An image can fit the label's nominal disk yet
  fail host preparation or updates. The advertised minimum machine can
  fit no default job when its host filesystem reports less than its
  nominal disk.
- **Needs:** A KVM image-build host, the current image tooling, and the
  area sizing that the orchestrator chose on 5 November 2026. The area
  holds the job disk, an image budget, and the XFS overhead measured after
  formatting. Subtract the image budget before the disk split. The host
  gets a quarter of the remainder, including XFS overhead. Job disk gets
  the other three quarters. The minimum machine has 250 GB of disk, and
  its default job reserves 80 GB. If the user chooses the
  [project quota](agent-xfs-image-project-quota.md), the measured budget
  also sets its limit. Start with:

  ```sh
  bash image/build.sh .context/image-build
  ```

  Read `artifacts/measurement.json` under that build directory. Use the
  same image and storage layout as the real agent.
- **Done when:** The report proves that the image and its update peak fit
  in the image budget on every advertised minimum machine and at intended
  full VM occupancy on larger machines. VM reservations stay available.
  It distinguishes usable from nominal capacity. The image budget setting
  uses the measured value. Any smaller
  image, changed label disk, changed minimum hardware, or changed update
  strategy goes to the user before its requirement or code changes.
