---
title: Choose a label size
description: Use the CPU and memory figures of a job to choose its label size.
---

Open a finished job in **Jobs** in the ciproxy dashboard. Find the **CPU
and memory** panel. The panel shows figures after the job ends if ciproxy
has samples for that job.

## Read the job figures

- **Average CPU** shows how many virtual CPUs (vCPU) were busy on average
  over the time that ciproxy measured.
- **Peak memory** shows the highest memory use that ciproxy measured.
- **Out of memory** counts the processes that Linux stopped for lack of
  memory. **None** means the samples show no such stops. macOS does not
  report this count.

The page shows measured CPU in vCPU and measured memory in GB. These
figures have one decimal place. The top of each meter and plot shows the
capacity of the VM that ran the job. The VM keeps some memory for its
kernel. A job can run out of memory below the top of the memory meter.

On Linux, ciproxy subtracts available memory from total memory. On macOS,
ciproxy counts app, wired, and compressed memory, as Activity Monitor
does. Do not compare memory figures between macOS and Linux.

## Find the step

The chart shows CPU and memory over time. It marks job steps and Linux
out-of-memory kills. Match a spike or a kill mark to the step number in
the chart.

The step table shows each step's number, name, duration, average CPU,
and peak memory. On Linux, a row shows an out-of-memory count only if
Linux stopped a process for lack of memory during that step. A skipped
step or a step without samples shows no figures. If no step data is
available, the panel can still show job figures and a chart without
step marks.

Each figure covers the whole VM. A step's figures include the service
containers and processes that run during that step, even if an earlier
step started them.

## Choose a size

A Boat job can run on a VM larger than its label asks for. If the VM has
more vCPU or memory than the label, the corresponding meter and plot
mark the label size. Compare your figures with that mark as well as
the VM's capacity.

- If peak memory is close to the label size or the top of the meter,
  try a label with more memory. Also try more memory if Linux stopped a
  process for lack of memory.
- If a slow step keeps CPU near the top of its plot, try more vCPU.
  Compare the duration of that step after the change.
- If average CPU is far below the VM's capacity, check the chart for short
  periods of high CPU use before you try fewer vCPU. Make sure the smaller
  label still has enough memory for the job.

See [Labels and VM sizes](/docs/reference/labels/) for the available sizes.
Change `runs-on` to the label that you want to test. Run the same job
again and check its figures and duration. ciproxy does not change your
label based on these figures.
