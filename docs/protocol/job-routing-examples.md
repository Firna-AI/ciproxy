# Job routing worked examples

These examples apply the [slot and disk rules](job-routing.md#slots).
Disk and memory sizes use decimal GB.
They use a logical image root of 61,000,000,000 bytes.
The image already holds its own metadata and 1% root reserve.
The agent adds free job bytes and the 16 MiB kernel reserve pool.
It divides that sum by 0.97, rounds up, and adds the image root.
It rounds the complete root up to 4096 bytes.
This leaves an allowance for new ext4 metadata and reserved blocks.
The actual root changes when the image release changes.

| Label | Free job disk | Complete root bytes | Slots |
| --- | --- | --- | --- |
| `ciproxy` | 20 GB | 81,635,856,384 | 1 |
| `ciproxy-4vcpu` | 20 GB | 81,635,856,384 | 2 |
| `ciproxy-8vcpu` | 60 GB | 122,872,967,168 | 4 |

## Slots and the minimum machine

- An own machine with 16 threads and 64 GB keeps 4 threads and 16 GB for
  the host.
- Its other 12 threads and 48 GB each give 6 slots.
- Those slots offer 12 vCPU and 48 GB to jobs.
- An own machine with 4 threads and 16 GB keeps 1 thread and 4 GB for the
  host.
- Its remaining threads and memory each give 1.5 slots, which rounds down
  to 1 slot.
- At the unchanged 250 GB disk minimum, the 140 GB image budget leaves
  110 GB.
- The host keeps 27.5 GB, and the job disk is 82.5 GB.
- One default root uses 81,635,856,384 bytes.
- The machine cache can use the remaining 864,143,616 bytes.
- An image release with a larger root can reduce this machine's job fit.

## Six jobs on an AX42

- An AX42 with 16 threads and 64 GB has 6 slots.
- Its single usable 1024 GB filesystem leaves 884 GB after the image
  budget.
- The host keeps 221 GB, and the job disk is 663 GB.
- The job disk holds 8 default roots, and the slots hold 6 default jobs.
- So the AX42 runs 6 default jobs at a time.
- Six roots use 489,815,138,304 bytes.
- The machine cache can use the remaining 173,184,861,696 bytes.

## A mini PC limited by disk

- A mini PC with 16 threads, 64 GB, and a 512 GB disk has 6 slots.
- Its 140 GB image budget leaves 372 GB.
- Its host reserve keeps 93 GB, so its job disk is 279 GB.
- That job disk holds 3 default roots, while its slots hold 6 default jobs.
- So the mini PC runs 3 jobs of `ciproxy` at a time.
- Its 2 warm roots use 163,271,712,768 bytes.
- The machine cache can use the remaining 115,728,287,232 bytes.
- A third default root reduces that cache budget to 34,092,430,848 bytes.
- For each [default label](labels.md#default-labels), the mini PC runs
  these jobs at a time:

| Label | Slots | Free job disk | Complete root GB, rounded | By slots | By disk | At a time |
| --- | --- | --- | --- | --- | --- | --- |
| `ciproxy` | 1 | 20 GB | 81.636 | 6 | 3 | 3 |
| `ciproxy-4vcpu` | 2 | 20 GB | 81.636 | 3 | 3 | 3 |
| `ciproxy-8vcpu` | 4 | 60 GB | 122.873 | 1 | 2 | 1 |

[Job routing](job-routing.md) gives the fit and warm runner requirements.
