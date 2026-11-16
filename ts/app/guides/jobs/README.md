# Jobs pages

Jobs of the [dashboard](../../README.md) shows the live queue, History of
finished jobs, and a page for each job. The
[dashboard guide](../../README.md#pages) gives their addresses and their
query. The banner of these pages is under
[Jobs, Savings, and billing](../../README.md#jobs-savings-and-billing).

## The pages

- Jobs Live reads the queue again every 5 seconds while the tab shows. A
  failed refresh keeps the last queue and says so. A figure that the API
  leaves out says "Unknown", never 0. A waiting job shows its label, not a
  machine, because GitHub gives it to whichever runner is idle first.
- The notices of the queue promise a place only from the facts of each
  waiting label. A full queue counts the jobs whose label fits your
  machines. It promises Boat after the overflow wait only to the labels that
  Boat starts now, and Boat at once to a label that no own machine runs.
  While the Boat costs have no safe count, it promises no Boat start. While
  Boat takes no new job, a notice says that only the jobs whose label fits
  your machines wait there for a free slot.
- A saved Boat retry time holds new Boat starts only while it is after the
  browser time of the read, because the API gives no response time. A
  refused Boat size holds only the labels that use it. A label that no
  permitted place fits names its full request and each place that it
  forbids or that takes no jobs. It never names one short resource, because
  the API does not say which one is. It offers the largest Boat size only
  while the label allows Boat and waits when nothing fits.
- History asks for the finished jobs from the first UTC date of the chosen
  window, and shows every job that the API returns, newest first. The API
  gives no total, so the footer counts the jobs of the list and names the
  history window of the plan in days.
- The job page shows the request of the label as the API kept it, or
  "Requested size unknown", and never the current size of the label. It names
  the VM that ran the job, and why the job ran there when the API gives a
  reason. A job that no runner took is not a start.

## Key Code

- [src/screens/jobs](../../src/screens/jobs) holds Jobs, History, and the
  job page. [jobs-links.ts](../../src/view/jobs-links.ts) builds the Jobs
  addresses and the History query.
  [jobs-places.ts](../../src/view/jobs-places.ts) gives where the waiting
  jobs of each label can start, and the one hold of Boat at the time of the
  read, which every notice of the queue reads.
  [jobs-label-notices.ts](../../src/view/jobs-label-notices.ts) words the
  notices of each waiting label from those facts.

## Related docs

- [Job routing](../../../../docs/protocol/job-routing.md) says what the
  dashboard shows of each job, under "What the dashboard shows".
- The [dashboard scripts](../../scripts/README.md#browser-checks) drive
  these pages in a browser.
