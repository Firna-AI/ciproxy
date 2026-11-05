# Image budget and XFS sizing

The image budget comes out of the underlying host filesystem first.
The job disk is three quarters of the remaining disk.
The host keeps the other quarter for the system and XFS overhead.
The area file holds job disk, image budget, and measured XFS overhead.
The agent leaves the rest of the host reserve outside the area.
A new installation needs at least 250 GB of disk.

## Image budget setting

The default image budget is **140,000,000,000 bytes**, or 140 SI GB.
The full `ubuntu-latest` software uses 58.3 GB installed.
Its compressed download is 18.3 GB.
Two installed images and one download need about 135 GB during an update.
The budget also holds kernels, jail support, and image metadata.
Job disk remains separate from this budget.
An image that cannot fit at peak is refused before it consumes job space.
The existing usable base remains available after an oversized update.

Set an explicit byte count on first install:

```sh
sudo env CIPROXY_IMAGE_BUDGET_BYTES=160000000000 \
  /usr/local/bin/ciproxy-agent install --token TOKEN
```

The value must be an integer greater than 16,777,216 bytes.
The image budget is subtracted before the host and job disk split.
Measured overhead must leave system space in the host reserve.
The agent reports an actionable error when they do not fit.
The first durable area record includes the chosen budget.
Install, runtime recovery, downloads, extraction, and VM support use it.
The service can read it from the record without inheriting the shell.
An explicit environment value that differs from the record is an error.
The agent never silently ignores or changes that value.

To change an established budget, use the replacement steps below.
The current format does not resize an active or published area.

## Measured overhead and support space

Fresh install starts with the complete job disk plus the image budget.
It creates XFS with reflinks and mounts the owned file.
It creates the support directories before it measures overhead.
The measurement is backing length minus mounted available bytes.
This includes the XFS log, metadata, and reserved blocks.
The agent also keeps conservative filesystem support headroom:

- One thirty-second of the job disk permits later extent/COW metadata.
- Another 16 MiB covers small filesystem support allocations.

Both additions remain in the host reserve.
They increase filesystem headroom, not VM disk reservations.
The agent grows only this unpublished area and measures it again.
It publishes Ready only after all job bytes, image budget, and metadata
headroom remain available. Later install preserves that backing length.
No quota, quota mount option, or quota tool is required.

For a 250 GB host, the default image budget is 140 GB.
The remaining 110 GB splits into 82.5 GB of job disk and 27.5 GB for host.
The metadata headroom is about 2.59 GB plus measured empty-XFS costs.
The remaining host reserve stays outside the area for the system.
With a 160 GB image budget override, that same host has 67.5 GB of job disk.
That override leaves too little job disk for an 80 GB default runner.
These are logical capacities. A sparse file does not preallocate the host.
Unrelated host programs can still consume its physical free space.

The image allocator keeps a separate 16 MiB safety charge within the
configured image budget. Catalog replacement can use that bounded safety.
Each file and directory also receives a conservative support charge.
Each recognized VM jail keeps an 8 MiB support reservation in this budget.
This covers the future copied Firecracker executable and small jail files.
Actual copied executable, kernel, and config allocations are counted too.
Only root and Docker payload blocks at an actual VM root are excluded.
Their full logical disks already belong to the protocol job disk.
An arbitrary file with the same name elsewhere is still counted.

## Physical write enforcement

One allocator counts physical allocated blocks, not sparse logical length.
It counts retained bases, held bases, pending downloads, extracted files,
kernel copies, config files, jail support, and catalog support.
Concurrent writers share its outstanding reservations.

Every nonzero write reserves rounded filesystem blocks first.
The safe `rustix` fallocate API preallocates exactly that reserved range.
Only then does the write run. It cannot create speculative extents beyond
that range. A refused reservation reaches neither allocation nor write.
Sparse zero runs keep holes and their exact final logical length.
Adjacent blocks are coalesced before reservation and allocation.

The active runtime owns image mutation and serializes fetches.
Recovery removes abandoned downloads and unused older bases before it
checks the current budget. Fetch repeats reclamation before admission.
Preparation holds protect their selected base while an update proceeds.
An update cannot evict a held base or the current usable base to fit.
It can instead fail and keep the current image.
Future fetches can retry after holds or other support allocations end.

## Old areas and budget replacement

The old version 1 area held only three quarters of the host filesystem.
It cannot provide the complete job disk together with nonzero images.
The new agent does not treat that old file as sufficient.
It preserves its data and gives a replacement instruction.

Use these steps for a version 1 area or a changed image budget:

1. Keep the existing agent binary available for recovery.
2. Drain the machine through that working agent.
3. Wait for its running jobs and VMs to finish.
4. Stop the agent service.
5. Back up the control state and the complete area file.
6. Disable the owned mount unit and unmount the area normally.
7. Keep the old area file and ownership record together in the backup.
8. Move those stopped files outside `/var/lib/ciproxy`.
9. Keep `state.json` in place to preserve machine identity and drain state.
10. Run install with the desired image budget.

The mountpoint must be empty before replacement.
Do not force or lazily unmount an area with remaining users.
The new daemon fetches a verified base when needed.
Existing image data can remain in the backup until recovery is complete.

New Creating records include a durable growth intent with old and target
lengths. Install resumes only those expected lengths and the same inode
and XFS UUID. It never reformats a valid filesystem during resume.
An unexplained size change still gives a non-destructive repair error.

## Disposable filesystem regression

The real regression is separate from fake-only unit tests and the default
gate. It needs root, loop-mount permission, XFS support, and the Linux tools
installed by host preparation. It uses a disposable 2 GiB ext4 file.
It needs about 2 GiB of temporary physical space and the Rust toolchain.
KVM, GitHub, and a registered agent are not needed.

From the repository root, with Cargo available in the root environment:

```sh
sudo -E python3 crates/ciproxy-agent/scripts/storage-regression.py
```

The wrapper preflights resources and returns SKIP with exit 77 when they
are unavailable. It then runs this explicit opt-in target:

```sh
cargo test --locked --package ciproxy-agent \
  --features filesystem-regression --test storage-regression
```

The target imports production sizing, accounting, and bounded-write code.
It creates and measures the XFS area through that sizing calculation.
It writes a nonzero 128 MiB base through the production image writer.
It reflinks a VM root and overwrites the complete protocol job disk.
The base digest must stay unchanged.
An oversized image update must fail before any payload block is allocated.
The wrapper saves evidence under `.context/ciproxy-mvp/`.
It removes its mounts and files after the run.
It preserves a busy path and fails if safe cleanup is impossible.
