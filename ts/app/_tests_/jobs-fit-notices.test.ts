import assert from "node:assert/strict";
import test from "node:test";

import type { JobOverflow, WaitingLabelFit } from "../src/api/jobs-types.ts";
import type { ProviderHold } from "../src/api/schema.ts";
import { liveNotices } from "../src/view/jobs-notices.ts";
import { boatPlace } from "../scripts/m20-fixtures-labels.ts";
import {
  connectedBoat,
  fitting,
  largest,
  queue,
  readAt,
  waitingLabel,
} from "./jobs-notices-support.ts";

// The retry time of a Boat refusal shows in the time zone of the browser.
process.env.TZ = "UTC";

const nbsp = "\u00a0";
const request = "8 vCPU, 16 GB of memory, and 60 GB of free disk";
const largestSize = "Boat xlarge, with 16 vCPU and 64 GB";
const atLimit = {
  ...connectedBoat,
  counted_cents: "10000",
  limit_reached: true,
} satisfies JobOverflow;

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
      "No machine of yours and no Boat size fits a job of 2 vCPU, 8 GB of memory, and 500 GB of free disk. Change the label, or let it run on the largest size.",
    ],
  },
  {
    name: "a label whose order holds own machines only",
    label: waitingLabel("ciproxy-own", 2, "none", { state: "disabled" }),
    notice: [
      "fits-label-ciproxy-own",
      `No machine of yours fits a job of ${request}, and the label runs on your machines only. Change the label.`,
    ],
  },
  {
    name: "a label that does not run on own machines",
    label: waitingLabel("ciproxy-boat", 2, "forbidden", {
      state: "no_size_fits",
    }),
    notice: [
      "fits-label-ciproxy-boat",
      `The label does not run on your machines, and no Boat size fits a job of ${request}. Change the label, or let it run on the largest size.`,
    ],
  },
  {
    name: "a label with an empty order",
    label: waitingLabel("ciproxy-off", 2, "forbidden", { state: "disabled" }),
    notice: [
      "fits-label-ciproxy-off",
      "The order of the label has no place. Change the label.",
    ],
  },
  {
    name: "no own machine that takes jobs, and no provider in the order",
    label: waitingLabel("ciproxy-own", 2, "no-machine", { state: "disabled" }),
    notice: [
      "fits-label-ciproxy-own",
      "None of your machines takes jobs now, and the label runs on your machines only. Change the label.",
    ],
  },
  {
    name: "a label that already runs on the largest Boat size",
    label: waitingLabel(
      "ciproxy-big",
      2,
      "none",
      { state: "no_size_fits" },
      { whenNothingFits: "largest_overflow" },
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
      "largest-boat-label-ciproxy-big",
      `No machine of yours and no Boat size fits a job of ${request}, so ciproxy starts them on ${largestSize}.`,
    ],
  },
  {
    name: "the largest Boat size while no own machine takes jobs",
    label: waitingLabel("ciproxy-big", 2, "no-machine", largest),
    notice: [
      "largest-boat-label-ciproxy-big",
      `None of your machines takes jobs now, and no Boat size fits a job of ${request}, so ciproxy starts them on ${largestSize}.`,
    ],
  },
  {
    name: "the largest Boat size for a label that does not run on own machines",
    label: waitingLabel("ciproxy-big", 2, "forbidden", largest),
    notice: [
      "largest-boat-label-ciproxy-big",
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

test("a label that waits for a hold of a place that fits it gives that reason once", () => {
  const offLabel = waitingLabel("ciproxy-16vcpu", 2, "forbidden", fitting, {
    order: [boatPlace],
    hold: "off",
  });
  const off = liveNotices(
    { ...queue([offLabel], connectedBoat, 2), overflow_enabled: false },
    "/settings?workspace=w",
    readAt,
  );
  assert.deepEqual(off, [
    {
      key: "held-label-ciproxy-16vcpu",
      tone: "warning",
      title: "2 jobs with the label ciproxy-16vcpu wait",
      body: "Overflow providers are off. No other place of its order can take them now.",
      action: {
        kind: "go",
        label: "Turn on overflow",
        href: "/settings?workspace=w",
      },
    },
  ]);
  const largestHeld = waitingLabel("ciproxy-32vcpu", 2, "none", largest, {
    hold: "spend_limit",
  });
  assert.deepEqual(
    liveNotices(
      queue([largestHeld], atLimit, 2),
      "/settings?workspace=w",
      readAt,
    ).map((notice) => [notice.key, notice.body, notice.action]),
    [
      [
        "held-label-ciproxy-32vcpu",
        `Boat reached its spend limit. No other place of its order can take them now. Boat takes jobs again on 1${nbsp}December.`,
        { kind: "go", label: "Raise the limit", href: "/settings?workspace=w" },
      ],
    ],
    "the reason of the provider that would run it, not that nothing fits",
  );
});

/** A label whose order holds only Boat, while Boat holds new starts. */
function boatOnly(hold: ProviderHold): WaitingLabelFit {
  return waitingLabel("ciproxy-boat", 2, "forbidden", fitting, {
    order: [boatPlace],
    hold,
  });
}

const holdEndCases: {
  name: string;
  label: WaitingLabelFit;
  overflow: JobOverflow;
  notices: [string, string][];
}[] = [
  {
    name: "a Boat-only queue at the Boat limit",
    label: boatOnly("spend_limit"),
    overflow: atLimit,
    notices: [
      [
        "held-label-ciproxy-boat",
        `Boat reached its spend limit. No other place of its order can take them now. Boat takes jobs again on 1${nbsp}December.`,
      ],
    ],
  },
  {
    name: "a Boat refusal whose retry time is after the read",
    label: boatOnly("retry_after"),
    overflow: { ...connectedBoat, retry_at: "2026-11-06T12:05:00Z" },
    notices: [
      [
        "held-label-ciproxy-boat",
        "Boat refuses new sandboxes. No other place of its order can take them now. ciproxy tries Boat again at 12:05.",
      ],
    ],
  },
  {
    name: "a Boat refusal whose retry time passed before the read",
    label: boatOnly("retry_after"),
    overflow: { ...connectedBoat, retry_at: "2026-11-06T11:59:00Z" },
    notices: [["machines-full", "2 jobs start on Boat at once."]],
  },
];

test("a label that waits for a held provider gives when it takes jobs again, and no refusal whose retry time passed before the read", () => {
  assert.deepEqual(
    holdEndCases.map((item) => [
      item.name,
      liveNotices(
        queue([item.label], item.overflow),
        "/settings?workspace=w",
        readAt,
      ).map((notice) => [notice.key, notice.body]),
    ]),
    holdEndCases.map((item) => [item.name, item.notices]),
  );
});
