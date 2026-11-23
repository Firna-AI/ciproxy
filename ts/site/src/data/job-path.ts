// The text of the picture on the docs page How ciproxy works, as the docs
// mockup gives it: the parts of ciproxy and the path of one job, from the
// queue in GitHub to the end of its VM, and the other way of a job to the
// Boat account. The numbers are the steps of "A job on your machines" on the
// same page, so that the list explains the picture. The picture shows no
// setting values, such as the overflow wait. The text of the page keeps them.

/** The part that a box shows. It also names the place of the box. */
export type JobPathPart = "github" | "ciproxy" | "machine" | "runner" | "boat";

export type JobPathBox = {
  part: JobPathPart;
  /** The number of the step in "A job on your machines". Boat has none. */
  step?: number;
  title: string;
  text: string;
};

export type JobPath = {
  /** The path of a job on your machines, in the order of its steps. */
  steps: readonly JobPathBox[];
  /** The other way of a job: from ciproxy to your Boat account. */
  boat: JobPathBox;
  /**
   * When a job takes the other way. It stays true for a macOS job, which
   * never goes to Boat.
   */
  boatWhen: string;
  caption: string;
};

export const jobPath: JobPath = {
  steps: [
    {
      part: "github",
      step: 1,
      title: "GitHub",
      text: "Queues a job of a ciproxy label.",
    },
    {
      part: "ciproxy",
      step: 2,
      title: "ciproxy",
      text: "Decides how many runners start, and where.",
    },
    {
      part: "machine",
      step: 3,
      title: "Your machine",
      text: "The agent starts a fresh VM with a runner.",
    },
    {
      part: "runner",
      step: 4,
      title: "The runner",
      text: "Takes a job from GitHub and runs it. Then the VM ends.",
    },
  ],
  boat: {
    part: "boat",
    title: "Your Boat account",
    text: "Runs the job in its own sandbox.",
  },
  boatWhen: "Or, when your machines have no room and Boat can take the job",
  caption: "A job on ciproxy, from the queue in GitHub to the end of its VM.",
};
