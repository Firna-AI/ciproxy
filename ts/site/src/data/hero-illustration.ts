// The example values of the product picture in the hero of the landing page:
// the Jobs page of an example workspace at a release peak, as the landing
// mockup shows it. They are fixed values of an illustration, not the data of
// a workspace, and the page labels the picture as an illustration. Each
// machine is a Hetzner AX42 with 6 slots, and every slot runs a job.

export type SlotState = "busy" | "warm" | "free";

/** Where a running job runs: an own machine, or Boat. */
export type JobPlace = { machine: string } | "boat";

export type IllustrationJob = {
  name: string;
  repo: string;
  ref: string;
  place: JobPlace;
  time: string;
};

const fullMachine: readonly SlotState[] = Array<SlotState>(6).fill("busy");

export const heroIllustration = {
  workspace: "mokly-ai",
  initial: "M",
  machines: [
    { name: "hetzner-ax42-1", slots: fullMachine },
    { name: "hetzner-ax42-2", slots: fullMachine },
    { name: "hetzner-ax42-3", slots: fullMachine },
  ],
  onBoat: 46,
  running: 64,
  waiting: 12,
  /** How long a job waits for a slot before it starts on Boat. */
  boatWaitSeconds: 30,
  boatSpent: 61.2,
  boatLimit: 200,
  jobs: [
    {
      name: "build (linux-x64)",
      repo: "mokly",
      ref: "v0.19.0",
      place: { machine: "hetzner-ax42-1" },
      time: "3m 02s",
    },
    {
      name: "build (linux-arm64)",
      repo: "mokly",
      ref: "v0.19.0",
      place: { machine: "hetzner-ax42-2" },
      time: "3m 01s",
    },
    {
      name: "e2e (chromium) 1/4",
      repo: "mokly",
      ref: "#1544",
      place: "boat",
      time: "1m 12s",
    },
    {
      name: "e2e (chromium) 2/4",
      repo: "mokly",
      ref: "#1544",
      place: "boat",
      time: "1m 12s",
    },
    {
      name: "test (rust)",
      repo: "mokly",
      ref: "#1544",
      place: { machine: "hetzner-ax42-3" },
      time: "2m 40s",
    },
    { name: "clippy", repo: "mokly", ref: "#1544", place: "boat", time: "58s" },
    {
      name: "unit (node 24)",
      repo: "mokly-cloud",
      ref: "#319",
      place: "boat",
      time: "44s",
    },
  ] satisfies readonly IllustrationJob[],
} as const;

/** The jobs that a phone shows, so that the picture stays short. */
export const phoneJobCount = 4;
