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
- The live queue shows the running jobs on each provider that is connected
  or runs jobs now, as "On Boat".
- The notices of the queue promise a place only from the facts of each
  waiting label. A job goes to the first place of its label order that can
  take it. A full queue counts the jobs whose label fits your machines. It
  promises a provider after the overflow wait only to the labels that the
  provider starts now, and a provider at once to a label that no own machine
  runs or that lists the provider before own machines. While the costs of
  that provider have no safe count, it promises no start there. While a
  provider takes no new job, its notice names it, and says that only the
  jobs whose label fits your machines wait there for a free slot, or that
  jobs skip it when another provider starts them.
- A saved retry time of a provider holds new starts there only while it is
  after the browser time of the read, because the API gives no response
  time. A refused size holds only the labels that use it. A label that no
  place of its order fits names its full request and each place of the
  order. It never names one short resource, because the API does not say
  which one is. It offers the largest size only while the order holds a
  provider and the label waits when nothing fits.
- When the routing of the API says that a label waits for a place that fits
  it, the notice of the label gives the reason of that place, such as
  "Overflow providers are off" or the spend limit of a provider, as the
  label list does. A label with "Run on the largest size" gives the reason
  of the provider that would run it. A hold that a notice of the provider
  states for every job shows once.
- History asks for the finished jobs from the first UTC date of the chosen
  window, and shows every job that the API returns, newest first. The API
  gives no total, so the footer counts the jobs of the list and names the
  history window of the plan in days. The place filter offers All, Own
  machines, and one choice for each provider of ciproxy, and its choices
  wrap on a narrow screen. The API does not say which providers ran jobs in
  the window, so the filter offers each provider.
- The job page shows the request of the label as the API kept it, or
  "Requested size unknown", and never the current size of the label. It names
  the VM that ran the job, the provider and the size that ran it on a
  provider, and why the job ran there when the API gives a reason. A job
  that no runner took is not a start.

## Key Code

- [src/screens/jobs](../../src/screens/jobs) holds Jobs, History, and the
  job page. [jobs-links.ts](../../src/view/jobs-links.ts) builds the Jobs
  addresses and the History query.
  [jobs-places.ts](../../src/view/jobs-places.ts) gives where the waiting
  jobs of each label can start, and the hold of each provider at the time
  of the read, which every notice of the queue reads.
  [jobs-label-notices.ts](../../src/view/jobs-label-notices.ts) words the
  notices of each waiting label from those facts, and
  [jobs-provider-notices.ts](../../src/view/jobs-provider-notices.ts) the
  notices of each provider hold.

## Related docs

- [Job routing](../../../../docs/protocol/job-routing.md) says what the
  dashboard shows of each job, under "What the dashboard shows".
- The [dashboard scripts](../../scripts/README.md#browser-checks) drive
  these pages in a browser.
