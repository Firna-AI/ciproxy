# Job routing worked examples

These examples apply the [slot and disk rules](job-routing.md#slots).
Disk and memory sizes use decimal GB.

## Slots and the minimum machine

- An own machine with 16 threads and 64 GB, such as a Hetzner AX42, keeps
  4 threads and 16 GB as its host reserve.
- Its other 12 threads give 6 slots, and its other 48 GB give 6 slots.
- So it has 6 slots, which offer 12 vCPU and 48 GB to jobs.
- An own machine with 4 threads and 16 GB keeps 1 thread and 4 GB as its
  host reserve.
- Its other 3 threads give 1.5 slots, and its other 12 GB give 1.5 slots.
- Both numbers round down, so it has 1 slot.
- So every own machine that meets the
  [hardware minimum](agent.md#install) of 4 cores and 16 GB has at least
  1 slot.
- At the 250 GB disk minimum, the 140 GB image budget leaves 110 GB.
- The host keeps 27.5 GB, and the job disk is 82.5 GB.
- One default `ciproxy` job holds 80 GB, leaving 2.5 GB for the machine
  cache.

## A mini PC limited by disk

- A mini PC with 16 threads, 64 GB, and a 512 GB disk has 6 slots, as the
  AX42 has.
- Its 140 GB image budget leaves 372 GB.
- Its host reserve keeps 93 GB, so its job disk is 279 GB.
- A job of the label `ciproxy` holds 80 GB, so the job disk holds 3 such
  jobs at a time, and the slots hold 6.
- So the mini PC runs 3 jobs of `ciproxy` at a time.
- While its 2 warm runners wait for jobs, they hold 160 GB, and the machine
  cache can use the other 119 GB.
- When a third job VM of `ciproxy` starts, the machine cache frees space
  until it uses at most 39 GB.
- For each [default label](labels.md#default-labels), the mini PC runs
  these jobs at a time:

| Label | Slots of a job | Disk of a job | By slots | By disk | At a time |
| --- | --- | --- | --- | --- | --- |
| `ciproxy` | 1 | 80 GB | 6 | 3 | 3 |
| `ciproxy-4vcpu` | 2 | 80 GB | 3 | 3 | 3 |
| `ciproxy-8vcpu` | 4 | 120 GB | 1 | 2 | 1 |

[Job routing](job-routing.md) gives the fit and warm runner requirements.
