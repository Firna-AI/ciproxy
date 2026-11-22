// What runs in each busy slot of hetzner-ax42-1 on the busy afternoon.

export type SlotJob = { name: string; context: string; time: string };

export const ax42OneSlotJobs: readonly SlotJob[] = [
  { name: "test (rust)", context: "mokly #1542", time: "4m 12s" },
  { name: "clippy", context: "mokly #1542", time: "2m 05s" },
  { name: "visual diff", context: "mokly #1539", time: "52s" },
  { name: "build (linux-x64)", context: "mokly-cloud main", time: "3m 30s" },
  { name: "e2e (webkit) 1/2", context: "mokly #1540", time: "5m 01s" },
];

/** What runs in each busy slot of the Macs in the what-ifs of macOS jobs. */
export const macSlotJobs: readonly SlotJob[] = [
  { name: "test (safari)", context: "mokly #1542", time: "6m 40s" },
  { name: "build (macos-arm64)", context: "mokly main", time: "9m 03s" },
];
