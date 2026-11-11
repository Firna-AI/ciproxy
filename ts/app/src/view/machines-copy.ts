// Every word of the Machines pages, after the approved mockups: the list, the
// machine page, Add a machine, and the Overflow panel. Screens and models
// take their text from here, so one change of a word changes every place
// that shows it. Values arrive formatted.

import { dotJoin, formatExactCount, unbroken } from "./machines-format.ts";

const count = (value: number, one: string, many: string) =>
  unbroken(`${value} ${value === 1 ? one : many}`);

export const listCopy = {
  title: "Machines",
  addMachine: "Add machine",
  machines: "Machines",
  slotsInUse: "Slots in use",
  warmRunners: "Warm runners",
  cacheHits: "Cache hits, 7 days",
  noRestores: "No restores yet",
  ownMachines: "Own machines",
  columns: {
    machine: "Machine",
    health: "Health",
    slots: "Slots",
    cache: "Cache",
    seen: "Last seen",
  },
  legend: { job: "Running a job", warm: "Warm runner", free: "Free" },
  emptyTitle: "Add your first machine",
  emptyBody:
    "Run one command on a Linux machine. Jobs start there first, and on Boat when it is busy.",
  loading: "Loading machines…",
  subject: "machines",
  refreshTitle: "Unable to refresh machines",
  refreshBody:
    "Machines shows the last state that it read, and tries again by itself.",
  tryAgain: "Try again",
  openMachine: "Open machine",
  openMachines: "Open Machines",
  slotsOf: (used: number, total: number) => `${used} of ${total}`,
} as const;

export const statusWords = {
  healthy: "Healthy",
  draining: "Draining",
  offline: "Offline",
  "no-slots": "No slots",
} as const;

export const slotCopy = {
  offline: "No slots available",
  none: "No slots",
  busyWarm: (busy: number, warm: number) =>
    dotJoin(unbroken(`${busy} busy`), unbroken(`${warm} warm`)),
  unreported: (slots: number) => count(slots, "slot", "slots"),
  row: (index: number) => `Slot ${index}`,
  job: "Running a job",
  warm: "Warm runner, ready",
  free: "Free",
  freeNoDisk: "Free, no disk for a job",
  size: (slots: number) =>
    `${count(slots, "slot", "slots")} of ${unbroken("2 vCPU")} and ${unbroken("8 GB")}. A larger VM size takes several slots.`,
  fits: (jobs: number) =>
    `${count(jobs, "job", "jobs")} of ciproxy ${jobs === 1 ? "fits" : "fit"} at a time.`,
  offlineBody: "Slots show again when the machine is back.",
  noSlotsBody: (elsewhere: string) => `No slots. ${elsewhere}`,
  unreportedBody: "The slots show when the machine reports them.",
} as const;

/** Where jobs go while a machine takes none: "Jobs" or "New jobs" first. */
export const elsewhereCopy = {
  jobs: "Jobs",
  newJobs: "New jobs",
  machinesAndBoat: (subject: string) =>
    `${subject} run on your other machines and on Boat.`,
  machines: (subject: string) => `${subject} run on your other machines.`,
  boat: (subject: string) => `${subject} run on Boat.`,
  wait: (subject: string) => `${subject} wait until a machine can take them.`,
} as const;

export const detailCopy = {
  crumb: "Machines",
  drain: "Drain",
  remove: "Remove",
  stopDraining: "Stop draining",
  removeMachine: "Remove machine",
  notFound: "This machine is not in the workspace",
  disk: (size: string) => unbroken(`${size} GB disk`),
  slots: "Slots",
  jobHistory: "Job history",
  warmRunners: "Warm runners",
  warmCaption: "Ready for the next job when a slot is free",
  fewer: "Fewer warm runners",
  more: "More warm runners",
} as const;

export const noticeCopy = {
  drainingTitle: "Draining",
  draining: (jobs: number | null) => {
    const rest = "The machine takes no new jobs, and it keeps no warm runners.";
    if (jobs === null) return rest;
    if (jobs === 0) return `Its jobs have finished. ${rest}`;
    const verb = jobs === 1 ? "is" : "are";
    return `${count(jobs, "job", "jobs")} ${verb} finishing. ${rest}`;
  },
  offlineTitle: (host: string) => `${host} is offline`,
  lostContact: (since: string | null) =>
    since === null
      ? "ciproxy lost contact with it."
      : `ciproxy lost contact with it ${since} ago.`,
  offlineCheck: "Check that the machine is on and can reach ciproxy.dev.",
  noKvmTitle: (host: string) => `${host} cannot start VMs`,
  noKvm:
    "ciproxy runs each job in its own VM, which needs KVM. Turn on virtualization for this machine, then run the command again.",
  showCommand: "Show install command",
  noSlotsTitle: (host: string) => `${host} has no slots`,
  noSlots:
    "Its threads and memory leave no room for a job after the host reserve. Use a machine with more threads and memory.",
  diskLimitTitle: (host: string, jobs: number) =>
    `Disk limits ${host} to ${count(jobs, "job", "jobs")} of ciproxy at a time`,
  diskLimit: (vmDisk: string | null, jobDisk: string | null) =>
    vmDisk === null || jobDisk === null
      ? "Add disk to run more jobs at once."
      : `With the current image, a job of ciproxy holds ${unbroken(`${vmDisk} GB`)} of disk. The machine has ${unbroken(`${jobDisk} GB`)} for jobs after the VM images and the host reserve. Add disk to run more jobs at once.`,
  savedOffline: (host: string) =>
    `${host} gets the change when it connects again.`,
  savedTitle: "Saved",
  clearing: (host: string) => `${host} is clearing its cache`,
} as const;

export const failureCopy = {
  drain: (host: string) => `Unable to drain ${host}`,
  resume: (host: string) => `Unable to stop draining ${host}`,
  remove: (host: string) => `Unable to remove ${host}`,
  "clear-cache": (host: string) => `Unable to clear the cache of ${host}`,
  warm: "Unable to change the warm runners",
  network: "Check the connection, then try again.",
  other: "Try again.",
} as const;

export const healthCopy = {
  title: "Health",
  lastContact: "Last contact",
  noContact: "No contact yet",
  kvm: "KVM",
  kvmReady: "Ready",
  kvmMissing: "Not available",
  disk: "Disk",
  diskAt: (path: string) => `Disk ${path}`,
  diskFree: (free: string) => `${unbroken(`${free} GB`)} free`,
  noReport: "No report yet",
  agent: "Agent",
  upToDate: (version: string) => dotJoin(version, "up to date"),
  behind: (version: string, latest: string) =>
    dotJoin(version, unbroken(`${latest} available`)),
  oldValues: (since: string | null) =>
    since === null
      ? "KVM, disk, and agent show the values of the last contact."
      : `KVM, disk, and agent show the values of the last contact, ${since} ago.`,
} as const;

export const cacheCopy = {
  title: "Cache",
  clear: "Clear cache",
  disk: "Cache disk",
  used: (used: string, capacity: string) => `${used} of ${capacity} GB`,
  hits: "Cache hits, 7 days",
  docker: "Docker disks",
  dockerUse: (repositories: string, size: string) =>
    dotJoin(
      unbroken(
        `${formatExactCount(repositories)} ${repositories === "1" ? "repository" : "repositories"}`,
      ),
      unbroken(`${size} GB`),
    ),
  unreported: "Cache use shows after the machine reports it.",
  noReport: "No report yet",
  dialogTitle: (host: string) => `Clear the cache of ${host}?`,
  dialogBody: (used: string | null) =>
    `ciproxy deletes ${used === null ? "the cache entries" : `the ${unbroken(`${used} GB`)} of cache entries`} on this machine. The GitHub cache keeps its entries, so the next jobs restore from there.`,
  cancel: "Cancel",
} as const;

export const costCopy = {
  title: "Cost",
  edit: "Edit",
  none: "No monthly cost yet",
  uses: "The savings report uses this cost.",
  needs: "The savings report needs the monthly cost of each machine.",
  field: "Monthly cost in US dollars",
  invalid: "Enter an amount in US dollars, such as 53.40.",
  failed: (reason: string) => `Unable to save the cost. ${reason}`,
  save: "Save",
  cancel: "Cancel",
} as const;

export const overflowCopy = {
  title: "Overflow",
  settings: "Overflow settings",
  boat: "Boat",
  connected: (account: string, wait: string) =>
    dotJoin(
      `Connected to the account ${account}`,
      wait === "At once"
        ? "jobs start there at once"
        : `jobs start there after ${wait}`,
    ),
  machinesOnly: "Jobs run on your machines only.",
  notConnected: "Boat is not connected. Jobs run on your machines only.",
  off: "Overflow is off, so jobs run on your machines only.",
  rejected: (day: string) =>
    `Boat rejected the API key on ${day}. Jobs run on your machines only until you connect Boat with a new key.`,
  limit: (limit: string | null, nextMonth: string) =>
    `Boat spend reached ${limit === null ? "its limit" : `the ${limit} limit`}. Jobs run on your machines only. Boat takes jobs again on ${nextMonth}.`,
  spent: "Spent this month",
  spentOf: (spent: string, limit: string) => `${spent} of ${limit}`,
  pending: (amount: string) => `${amount} still pending`,
  noSpend: "Boat spend shows here when Boat reports it.",
  loading: "Loading overflow…",
  failed: "Unable to load overflow",
} as const;

export const addCopy = {
  title: "Add a machine",
  lead: (workspace: string) =>
    `Run this command on the new machine. It installs the ciproxy agent and connects the machine to ${workspace}.`,
  waiting: "Waiting for the machine to connect…",
  connected: (host: string) => `${host} connected`,
  diskHolds: (jobs: number) =>
    `Its disk holds ${count(jobs, "job", "jobs")} of ciproxy at a time.`,
} as const;
