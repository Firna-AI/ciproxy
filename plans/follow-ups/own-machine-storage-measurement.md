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
  allocations as well as the guest's root size. A nominal 80 GB device
  can report less usable filesystem capacity than the protocol example.
- **Task:** Extend the existing image and host measurement to record
  logical root size, allocated sparse bytes, archive size, update peak,
  per-VM kernel/runtime support files, and filesystem overhead. Record
  nominal device capacity, selected filesystem capacity, target area
  length, and mounted usable capacity separately. Compare the results
  with the selected image share and the default VM's full reservation.
- **Impact if no action:** An image can fit the label's nominal disk yet
  fail host preparation or updates. The advertised minimum machine can
  fit no default job once filesystem geometry is included.
- **Needs:** A KVM image-build host, the current image tooling, and the
  selected [area budget policy](agent-xfs-image-reserve.md). Start with:

  ```sh
  bash image/build.sh .context/image-build
  ```

  Read `artifacts/measurement.json` under that build directory. Use the
  same image and storage layout as the real agent.
- **Done when:** The report proves that the image and its update peak fit
  on every advertised minimum machine while VM reservations remain
  available. It distinguishes usable from nominal capacity. Any smaller
  image, changed label disk, changed minimum hardware, or changed update
  strategy goes to the user before its requirement or code changes.
