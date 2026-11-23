# The macOS image

This page states the job image of macOS VMs. [Agent images](agent-images.md)
states the Linux image and the terms of both images.

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
  [Job isolation on a Mac](security-isolation.md#job-isolation-on-a-mac)
  says why this is safe.
- Then the build adds the runner and the guest, as for the machine image.
- ciproxy serves each release of the macOS image from a private store. A
  URL without a valid signature cannot read the store.
- The control plane offers the macOS image only to machines of the
  [platform](agent.md#platforms) `macos_arm64`. A Linux machine never gets
  it, and a Mac never gets the machine image.
- For each download, the control plane gives the Mac a new URL.
- That URL names one object of the release and allows only a read of it.
- That URL expires 2 hours after the control plane makes it.
- The agent checks the byte size and the SHA-256 that come with the URL,
  as for the machine image.
- The control plane reads the macOS release only from an `https://` store
  address, because each read carries a signature in its URL.
- Each release file that the control plane reads, `manifest.json` and
  `downloads.json`, holds at most 1 MiB. A larger file fails the read.
- When no macOS release is set, or the control plane cannot read it, Macs
  get no image. Linux machines and Boat jobs are not affected.
- **Decision:** a private store, and a signed read URL for each download.
  Rejected: a public store on a custom domain, because anyone with the path
  could then download Xcode. Rejected: a Worker that checks a token in
  front of the store, because it is one more service to deploy and run.
  Reason: the user chose this on 23 November 2026.
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
- A server error status or a rate limit answer does not end the download
  command. The agent asks again a few times, after the wait that a rate
  limit answer names, but at most 60 s.
- **Decision:** the agent asks again after a server error or a rate limit
  answer in the same download command. Rejected: each HTTP error status
  ends the command, because the next command then hashes up to 60 GB again.
  Reason: the parent workspace chose this with the user's authority on 23
  November 2026.
- Another HTTP error status, such as the answer to an expired URL, fails
  the download command and keeps the bytes. The next download command
  brings a fresh URL.
- When the host cannot continue the bytes, for example because it does not
  serve the requested range, the agent starts the download again from zero.
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

## Related docs

- [Agent images](agent-images.md): the Linux image, its two builds, and the
  terms of each image.
- [Job isolation on a Mac](security-isolation.md#job-isolation-on-a-mac):
  why a guest without SIP is safe.
