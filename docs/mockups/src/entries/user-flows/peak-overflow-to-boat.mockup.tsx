import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";
import { formatDay, formatMoney } from "../../fixtures/format.ts";
import { releaseDay } from "../../fixtures/live-jobs.ts";
import { countSlots, machineScenes } from "../../fixtures/machines.ts";
import { boat, overflow } from "../../fixtures/overflow.ts";
import {
  providerStarts,
  savings,
  savingsPeriod,
  yourCost,
} from "../../fixtures/savings.ts";
import { liveScenes } from "../../screens/jobs/live-scenes.ts";

const steady = machineScenes.steady;
const busySlots = countSlots(steady, "busy");
const allSlots = steady.reduce((sum, machine) => sum + machine.slots.length, 0);
const overflowStarts = providerStarts(savings.providers);
const boatShare = Math.round(
  (overflowStarts / (savings.own.starts + overflowStarts)) * 100,
);
const savedOnBlacksmith = savings.alternatives.blacksmith - yourCost;

export const peakOverflowToBoat = defineUseCase({
  title: "A peak in which jobs overflow to Boat",
  description: `mokly-ai on Team with three own machines and Boat as its one provider. The order of each label is the own machines, then Boat. A release on ${formatDay(releaseDay)} fills every own slot; ciproxy starts runners on Boat for the jobs that wait, and the savings report of the last 30 days, which end that day, shows what the overflow cost.`,
  relatedDocs: related(
    "jobRouting",
    "overflow",
    "overflowSpend",
    "labels",
    "metering",
  ),
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
      description: `${busySlots} of ${allSlots} slots are busy, and only a few warm runners remain. The Overflow panel shows Boat, the one connected provider.`,
    },
    {
      screenPath: paths.jobsPeak,
      title: "The release peak overflows",
      description: `Every own slot is busy and ${liveScenes.peak.waiting} jobs wait, each with its label. Each job that waits ${overflow.waitSeconds} seconds goes to Boat, the next place of its label, and gets a runner there.`,
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
      description: `The last 30 days, ${savingsPeriod}, include the release. Boat took ${boatShare}% of the starts, in the Boat row of the cost table. With the own machines and Team, the report shows ${formatMoney(yourCost)}, ${formatMoney(savedOnBlacksmith)} less than the same jobs on Blacksmith.`,
    },
    {
      screenPath: paths.overflow,
      title: "Tune the wait and the limit",
      description: `The wait for a free slot on the own machines and the spend limit on Boat, ${formatMoney(boat.spendLimit)}, decide how much of a peak goes to Boat.`,
    },
    {
      screenPath: paths.editLabel,
      title: "The order of a label",
      description:
        "Each label sets the order of its places. ciproxy-large runs on the own machines, then on Boat; an Admin can move a place, remove it, or add another provider.",
    },
  ],
});
