import assert from "node:assert/strict";
import test from "node:test";

import type {
  JobOverflow,
  JobsLivePage,
  WaitingLabelFit,
} from "../src/api/jobs-types.ts";
import { liveNotices } from "../src/view/jobs-notices.ts";
import { liveJobs } from "../scripts/fixtures-jobs-live.ts";
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

test("wait notices follow the flags of the API, and an unknown fit claims no fit", () => {
  const base = liveJobs("live", "viewer", null);
  const waiting = base.waiting_labels[0];
  assert.notEqual(waiting, undefined);
  const page: JobsLivePage = {
    ...base,
    can_manage: false,
    visibility: [
      {
        organization: "northstar-labs",
        suspended: true,
        removed: false,
        github_url:
          "https://github.com/organizations/northstar-labs/settings/installations/7",
      },
    ],
    waiting_labels: waiting
      ? [
          {
            ...waiting,
            label_id: "a",
            nothing_fits: true,
            can_edit: false,
            ask_admin: true,
          },
          {
            ...waiting,
            label_id: "b",
            nothing_fits: null,
            can_edit: true,
            ask_admin: false,
          },
        ]
      : [],
  };
  const notices = liveNotices(page, "/settings?workspace=w", readAt);
  assert.deepEqual(
    notices.map((notice) => [notice.key, notice.tone, notice.action]),
    [
      [
        "suspended-northstar-labs",
        "warning",
        {
          kind: "leave",
          label: "Open GitHub",
          href: "https://github.com/organizations/northstar-labs/settings/installations/7",
        },
      ],
      ["fits-a", "warning", { kind: "ask", text: "Ask an Admin" }],
      ["fit-unknown-b", "info", null],
    ],
  );
  assert.equal(notices[2]?.body.includes("No machine and no Boat size"), false);
});

const boatEveryOne =
  "3 jobs wait for a free slot. Each one starts on Boat when it has waited 30 seconds.";

const placeCases: {
  name: string;
  labels: WaitingLabelFit[];
  overflow: JobOverflow;
  notices: [string, string][];
}[] = [
  {
    name: "a label that does not run on Boat",
    labels: [
      waitingLabel("ciproxy", 3, "fits", fitting),
      waitingLabel("ciproxy-own", 2, "fits", { state: "disabled" }),
    ],
    overflow: connectedBoat,
    notices: [
      [
        "machines-full",
        "5 jobs wait for a free slot. 3 of them start on Boat when they have waited 30 seconds.",
      ],
    ],
  },
  {
    name: "a label that no Boat size fits",
    labels: [
      waitingLabel("ciproxy-wide", 4, "fits", { state: "no_size_fits" }),
    ],
    overflow: connectedBoat,
    notices: [["machines-full", "4 jobs wait for a free slot."]],
  },
  {
    name: "a label whose Boat facts the API leaves out",
    labels: [waitingLabel("ciproxy", 4, "fits", { state: "unavailable" })],
    overflow: connectedBoat,
    notices: [["machines-full", "4 jobs wait for a free slot."]],
  },
  {
    name: "Boat costs that have no safe count",
    labels: [
      waitingLabel("ciproxy", 3, "fits", fitting),
      waitingLabel("ciproxy-32vcpu", 1, "none", largest),
    ],
    overflow: { ...connectedBoat, counted_cents: null, limit_reached: null },
    notices: [
      [
        "machines-full",
        "3 jobs wait for a free slot. Boat starts no new sandbox until ciproxy can count its costs.",
      ],
    ],
  },
  {
    name: "the Boat limit while a label that only Boat runs waits",
    labels: [
      waitingLabel("ciproxy", 3, "fits", fitting),
      waitingLabel("ciproxy-16vcpu", 2, "none", fitting),
    ],
    overflow: { ...connectedBoat, counted_cents: "10000", limit_reached: true },
    notices: [
      [
        "spend-limit",
        "Jobs whose label fits your machines wait there for a free slot. Boat takes jobs again on 1 December.",
      ],
    ],
  },
  {
    name: "a label that only Boat runs",
    labels: [
      waitingLabel("ciproxy", 3, "fits", fitting),
      waitingLabel("ciproxy-16vcpu", 2, "none", fitting),
    ],
    overflow: connectedBoat,
    notices: [
      [
        "machines-full",
        "3 jobs wait for a free slot. Each one starts on Boat when it has waited 30 seconds. 2 jobs start on Boat at once.",
      ],
    ],
  },
  {
    name: "a retry time that passed before the read",
    labels: [waitingLabel("ciproxy", 3, "fits", fitting)],
    overflow: { ...connectedBoat, retry_at: "2026-11-06T11:59:00Z" },
    notices: [["machines-full", boatEveryOne]],
  },
  {
    name: "a retry time equal to the read",
    labels: [waitingLabel("ciproxy", 3, "fits", fitting)],
    overflow: { ...connectedBoat, retry_at: "2026-11-06T12:00:00Z" },
    notices: [["machines-full", boatEveryOne]],
  },
  {
    name: "a retry time after the read",
    labels: [waitingLabel("ciproxy", 3, "fits", fitting)],
    overflow: { ...connectedBoat, retry_at: "2026-11-06T12:01:00Z" },
    notices: [
      [
        "boat-refused",
        "Waiting jobs take the first free slot on your machines, and ciproxy tries Boat again at 12:01.",
      ],
    ],
  },
  {
    name: "a refused size that no waiting label uses",
    labels: [waitingLabel("ciproxy", 3, "fits", fitting)],
    overflow: { ...connectedBoat, refused_sizes: ["xlarge"] },
    notices: [["machines-full", boatEveryOne]],
  },
  {
    name: "a refused size of a label that only Boat runs",
    labels: [
      waitingLabel("ciproxy", 3, "fits", fitting),
      waitingLabel("ciproxy-16vcpu", 2, "none", { ...fitting, refused: true }),
    ],
    overflow: { ...connectedBoat, refused_sizes: ["large"] },
    notices: [
      [
        "size-refused-label-ciproxy-16vcpu",
        "Its jobs wait, and ciproxy tries Boat again.",
      ],
      ["machines-full", boatEveryOne],
    ],
  },
  {
    name: "a refused size of a label that fits own machines",
    labels: [
      waitingLabel("ciproxy", 3, "fits", fitting),
      waitingLabel("ciproxy-8vcpu", 2, "fits", { ...fitting, refused: true }),
    ],
    overflow: { ...connectedBoat, refused_sizes: ["large"] },
    notices: [
      [
        "size-refused-label-ciproxy-8vcpu",
        "Its jobs take the first free slot on your machines, and ciproxy tries Boat again.",
      ],
      [
        "machines-full",
        "5 jobs wait for a free slot. 3 of them start on Boat when they have waited 30 seconds.",
      ],
    ],
  },
];

test("the live queue promises a free slot or a Boat start only from the confirmed facts of each waiting label", () => {
  assert.deepEqual(
    placeCases.map((item) => [
      item.name,
      liveNotices(
        queue(item.labels, item.overflow),
        "/settings?workspace=w",
        readAt,
      ).map((notice) => [notice.key, notice.body]),
    ]),
    placeCases.map((item) => [item.name, item.notices]),
  );
});

test("an idle queue shows a Boat refusal only while its retry time is after the read", () => {
  const idle = (retry: string) =>
    liveNotices(
      queue([], { ...connectedBoat, retry_at: retry }, 2),
      "/settings?workspace=w",
      readAt,
    ).map((notice) => [notice.key, notice.body]);
  assert.deepEqual(
    [idle("2026-11-06T11:58:00Z"), idle("2026-11-06T12:05:00Z")],
    [
      [],
      [
        [
          "boat-refused",
          "Waiting jobs take the first free slot on your machines, and ciproxy tries Boat again at 12:05.",
        ],
      ],
    ],
  );
});
