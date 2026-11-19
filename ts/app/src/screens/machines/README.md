# Machines pages

Machines shows the machines of a workspace in the
[dashboard](../../../README.md) on three pages: the list, one machine, and
Add a machine. The [dashboard guide](../../../README.md#pages) gives their
addresses.

## The pages

- Every role sees the list and each machine. Only an Admin adds, drains, or
  removes a machine, clears its cache, or sets its warm runners and its
  monthly cost. Another role on Add a machine goes to the list.
- The list shows the figures of the machines, a table with the slots,
  health, and cache of each machine, a notice for each offline machine, and
  the Overflow panel with the Boat spend of the month.
- The three pages share one read of `GET /workspaces/{id}/machines`. They
  read it again every 10 seconds, and Add a machine every 5 seconds. A
  machine that the read does not hold shows a not-found state. After Remove,
  the page of the machine goes back to the list.
- The Overflow panel reads `GET /workspaces/{id}/overflow/summary` each time
  the list opens, and with Try again, never on an interval. A failed first
  read shows "Unable to load overflow" in the panel. A failed refresh keeps
  the last summary under a warning with Try again. Both keep the machines on
  the page.
- Add a machine shows the install command of Setup after its first read of
  the machines. A machine that was not in that read is new, and the page
  shows what its agent reported.
- Each machine shows one status, in this order: Offline, No slots,
  Draining, Preparing, and Healthy. Offline and Preparing come from the
  `state` of the API, which routing uses too. A preparing machine is online
  and gets its first job image, so it takes no job yet.
- The list shows "Getting the job image" in place of the slot bar of a
  preparing machine. Its page says that jobs start on it when it is ready,
  and where jobs run until then. Its slots show their size, but no slot bar
  and no free slot. Add a machine shows it as connected, with a line about
  the image.
- Slots in use, Warm runners, and the jobs of `ciproxy` at a time count only
  a machine that the API calls ready, so never a preparing machine, also
  while it drains.
- The machine page shows the saved warm runner count in every state, also
  while the machine drains, is offline, or has no slots. The slots show the
  warm runners that are ready.
- A busy slot names the job that the API matched to it, with its repository,
  its pull request or branch, and how long it has run. The name links to
  `/job?workspace=<id>&job=<id>` with the dashboard ID of the job, a page
  that the Jobs pages own. A job that the API could not match links nowhere,
  and the job ID of a machine report never makes a link. Job history opens
  History of Jobs.
- The agent version says whether it is the newest release only from a fresh
  report, and only when ciproxy could read the newest release.
- Cache shows the count of repositories with a Docker disk on the machine
  and the space of these disks, from the last report, also while the machine
  is offline. A report of no Docker disks shows 0, and a machine with no
  report yet shows no cache figures. The page shows the reported Docker
  disks on every tier. On Free, new jobs get no Docker disk. Clear cache
  keeps the Docker disks, and they have no action of their own.
- The machine cache and the Docker disks of a report are each measured or
  unavailable, as
  [Machine figures](../../../../../docs/protocol/cache.md#machine-figures)
  says. Each unavailable figure shows Unknown, the cache disk then has no
  meter, and one note names the unknown parts. A measured part keeps its
  figures. Clear cache keeps its rule, and its confirmation then names no
  size.
- Cache hits on the list cover the machines with a measured machine cache,
  and a line under the figure names each other machine. With no measured
  machine, the figure says Unknown. The Cache column says Unknown for an
  unavailable machine cache.
- When the API says that the disk holds fewer jobs of `ciproxy` than the
  slots, the page names the limit, the VM disk of one such job, and the disk
  for jobs, and the free slots beyond the limit say that they have no disk
  for a job. Add a machine shows the same limit for a machine that joins.

## Key Code

- The [Machines screen](machines-screen.tsx) opens the list, one machine, or
  Add a machine, as [machines-routes.ts](../../view/machines-routes.ts) reads
  the query. The three pages share the
  [machines store](../../state/machines.ts).
  [machines-changes.ts](../../state/machines-changes.ts) runs the changes of
  an Admin, one at a time, and [machines-copy.ts](../../view/machines-copy.ts)
  holds the words.
- [machines.ts](../../view/machines.ts) gives the status of each machine,
  the slots that count, and where jobs go while a machine takes none.
  [image-line.tsx](image-line.tsx) is the line under a machine that gets its
  first job image, which Setup shows too.
- [machines-cache.ts](../../view/machines-cache.ts) words the cache figures
  of each status, and gives the cache hits of the list.
- [machine-endpoints.ts](../../api/machine-endpoints.ts) holds the routes of
  Machines on the one HTTP client.
- [machines-overflow.ts](../../state/machines-overflow.ts) reads the overflow
  summary of the Overflow panel as a page read of
  [page-store.ts](../../state/page-store.ts).

## Related docs

- The [install](../../../../../docs/protocol/agent.md#install),
  [health](../../../../../docs/protocol/agent.md#health), and
  [removal](../../../../../docs/protocol/agent.md#drain-and-remove) of a
  machine, its [slots](../../../../../docs/protocol/job-routing.md#slots),
  [Clear cache](../../../../../docs/protocol/cache.md#cache-scopes), and its
  [Docker disks](../../../../../docs/protocol/docker-disks.md#tiers-and-the-machine-page).
- The [dashboard scripts](../../../scripts/README.md#browser-checks) drive
  these pages in a browser.
