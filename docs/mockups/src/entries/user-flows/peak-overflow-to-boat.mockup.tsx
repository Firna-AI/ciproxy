import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";

export const peakOverflowToBoat = defineUseCase({
  title: "A peak in which jobs overflow to Boat",
  description:
    "mokly-ai on Team with three own machines. A release on 18 November fills every own slot; ciproxy starts runners on Boat for the jobs that wait, and the savings report shows what the overflow cost.",
  relatedDocs: [planDoc],
  steps: [
    {
      screenPath: paths.jobs,
      title: "A busy day on own machines",
      description:
        "Most jobs run on the three own machines. Warm runners take new jobs at once, so nothing waits.",
    },
    {
      screenPath: paths.machines,
      title: "The machines fill up",
      description:
        "15 of 18 slots are busy, and only a few warm runners remain.",
    },
    {
      screenPath: paths.jobsPeak,
      title: "The release peak overflows",
      description:
        "Every own slot is busy and 12 jobs wait. Each job that waits 30 seconds gets a runner on Boat.",
    },
    {
      screenPath: paths.jobBoat,
      title: "A job that ran on Boat",
      description:
        "The job detail says why it ran on Boat and that its cache came from the GitHub cache.",
    },
    {
      screenPath: paths.savings,
      title: "What the overflow cost",
      description:
        "Boat took a quarter of the starts. With the own machines and Team, the month still costs less than half of Blacksmith.",
    },
    {
      screenPath: paths.overflow,
      title: "Tune the wait and the limit",
      description:
        "The wait before Boat and the monthly spend limit decide how much of a peak goes to Boat.",
    },
  ],
});
