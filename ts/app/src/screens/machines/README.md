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
- Add a machine shows the install command of Setup after its first read of
  the machines. A machine that was not in that read is new, and the page
  shows what its agent reported.
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
  report, and only when ciproxy could read the newest release. Cache shows
  the space of the Docker disks and their count of repositories.
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
- [machine-endpoints.ts](../../api/machine-endpoints.ts) holds the routes of
  Machines on the one HTTP client.

## Related docs

- The [install](../../../../../docs/protocol/agent.md#install),
  [health](../../../../../docs/protocol/agent.md#health), and
  [removal](../../../../../docs/protocol/agent.md#drain-and-remove) of a
  machine, its [slots](../../../../../docs/protocol/job-routing.md#slots),
  and [Clear cache](../../../../../docs/protocol/cache.md#cache-scopes).
- The [dashboard scripts](../../../scripts/README.md#browser-checks) drive
  these pages in a browser.
