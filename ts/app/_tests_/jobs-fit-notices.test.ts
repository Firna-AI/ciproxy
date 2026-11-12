import assert from "node:assert/strict";
import test from "node:test";

import type { WaitingLabelFit } from "../src/api/jobs-types.ts";
import { liveNotices } from "../src/view/jobs-notices.ts";
import {
  connectedBoat,
  largest,
  queue,
  readAt,
  waitingLabel,
} from "./jobs-notices-support.ts";

const request = "8 vCPU, 16 GB of memory, and 60 GB of free disk";
const largestSize = "Boat xlarge, with 16 vCPU and 64 GB";

const fitCases: {
  name: string;
  label: WaitingLabelFit;
  notice: [string, string];
}[] = [
  {
    name: "free disk that no place has",
    label: waitingLabel(
      "ciproxy-disk",
      2,
      "none",
      { state: "no_size_fits" },
      { size: { vcpus: 2, memory_gb: 8, disk_gb: 500 } },
    ),
    notice: [
      "fits-label-ciproxy-disk",
      "No machine of yours and no Boat size fits a job of 2 vCPU, 8 GB of memory, and 500 GB of free disk. Change the label, or let it run on the largest Boat size.",
    ],
  },
  {
    name: "a label that does not run on Boat",
    label: waitingLabel("ciproxy-own", 2, "none", { state: "disabled" }),
    notice: [
      "fits-label-ciproxy-own",
      `No machine of yours fits a job of ${request}, and the label does not run on Boat. Change the label.`,
    ],
  },
  {
    name: "a label that does not run on own machines",
    label: waitingLabel("ciproxy-boat", 2, "forbidden", {
      state: "no_size_fits",
    }),
    notice: [
      "fits-label-ciproxy-boat",
      `The label does not run on your machines, and no Boat size fits a job of ${request}. Change the label, or let it run on the largest Boat size.`,
    ],
  },
  {
    name: "a label that runs on neither place",
    label: waitingLabel("ciproxy-off", 2, "forbidden", { state: "disabled" }),
    notice: [
      "fits-label-ciproxy-off",
      "The label runs neither on your machines nor on Boat. Change the label.",
    ],
  },
  {
    name: "no own machine that takes jobs, and no Boat",
    label: waitingLabel("ciproxy-own", 2, "no-machine", { state: "disabled" }),
    notice: [
      "fits-label-ciproxy-own",
      "None of your machines takes jobs now, and the label does not run on Boat. Change the label.",
    ],
  },
  {
    name: "a label that already runs on the largest Boat size",
    label: waitingLabel(
      "ciproxy-big",
      2,
      "none",
      { state: "no_size_fits" },
      { whenNothingFits: "largest_boat" },
    ),
    notice: [
      "fits-label-ciproxy-big",
      `No machine of yours and no Boat size fits a job of ${request}. Change the label.`,
    ],
  },
  {
    name: "the largest Boat size beside own machines that do not fit",
    label: waitingLabel("ciproxy-big", 2, "none", largest),
    notice: [
      "largest-label-ciproxy-big",
      `No machine of yours and no Boat size fits a job of ${request}, so ciproxy starts them on ${largestSize}.`,
    ],
  },
  {
    name: "the largest Boat size while no own machine takes jobs",
    label: waitingLabel("ciproxy-big", 2, "no-machine", largest),
    notice: [
      "largest-label-ciproxy-big",
      `None of your machines takes jobs now, and no Boat size fits a job of ${request}, so ciproxy starts them on ${largestSize}.`,
    ],
  },
  {
    name: "the largest Boat size for a label that does not run on own machines",
    label: waitingLabel("ciproxy-big", 2, "forbidden", largest),
    notice: [
      "largest-label-ciproxy-big",
      `The label does not run on your machines, and no Boat size fits a job of ${request}, so ciproxy starts them on ${largestSize}.`,
    ],
  },
];

test("a fit notice names the permitted places and the full request, and never a short resource", () => {
  assert.deepEqual(
    fitCases.map((item) => [
      item.name,
      liveNotices(
        queue([item.label], connectedBoat, 2),
        "/settings?workspace=w",
        readAt,
      ).map((notice) => [notice.key, notice.body]),
    ]),
    fitCases.map((item) => [item.name, [item.notice]]),
  );
});
