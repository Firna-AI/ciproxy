import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";
import { formatMoney } from "../../fixtures/format.ts";
import {
  ax42,
  boat,
  countSlots,
  machineScenes,
} from "../../fixtures/machines.ts";
import { savings, yourCost } from "../../fixtures/savings.ts";
import { liveScenes } from "../../screens/jobs/live-scenes.ts";

const steady = machineScenes.steady;
const busySlots = countSlots(steady, "busy");
const allSlots = steady.length * ax42.slotsTotal;
const boatShare = Math.round(
  (savings.starts.boat / (savings.starts.own + savings.starts.boat)) * 100,
);
const savedOnBlacksmith = savings.alternatives.blacksmith - yourCost;

export const peakOverflowToBoat = defineUseCase({
  title: "A peak in which jobs overflow to Boat",
  description:
    "mokly-ai on Team with three own machines. A release on 18 November fills every own slot; ciproxy starts runners on Boat for the jobs that wait, and the savings report shows what the overflow cost.",
  relatedDocs: related("jobRouting", "overflow", "metering"),
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
      description: `${busySlots} of ${allSlots} slots are busy, and only a few warm runners remain.`,
    },
    {
      screenPath: paths.jobsPeak,
      title: "The release peak overflows",
      description: `Every own slot is busy and ${liveScenes.peak.waiting} jobs wait, each with its label. Each job that waits ${boat.waitSeconds} seconds gets a runner on Boat.`,
    },
    {
      screenPath: paths.jobBoat,
      title: "A job that ran on Boat",
      description:
        "The job detail says why it ran on Boat, names the Boat size that ran it, and shows that its cache came from the GitHub cache.",
    },
    {
      screenPath: paths.savings,
      title: "What the overflow cost",
      description: `Boat took ${boatShare}% of the starts. With the own machines and Team, the report shows ${formatMoney(yourCost)} for the month, ${formatMoney(savedOnBlacksmith)} less than the same jobs on Blacksmith.`,
    },
    {
      screenPath: paths.overflow,
      title: "Tune the wait and the limit",
      description:
        "The wait before Boat and the monthly spend limit decide how much of a peak goes to Boat.",
    },
  ],
});
