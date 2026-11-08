import assert from "node:assert/strict";
import test from "node:test";

import type {
  BoatLabelFit,
  JobOverflow,
  JobsLivePage,
  WaitingLabelFit,
} from "../src/api/jobs-types.ts";
import { liveNotices } from "../src/view/jobs-notices.ts";
import { fixtureJob } from "../scripts/fixtures-jobs.ts";
import { liveJobs } from "../scripts/fixtures-jobs-live.ts";

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
  const notices = liveNotices(page, "/settings?workspace=w");
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

/** A waiting label whose own machines fit or not, with its Boat fit. */
function waitingLabel(
  name: string,
  jobs: number,
  own: "fits" | "none",
  boat: BoatLabelFit,
): WaitingLabelFit {
  return {
    label_id: `label-${name}`,
    label_name: name,
    waiting_jobs: String(jobs),
    can_edit: true,
    ask_admin: false,
    nothing_fits: false,
    current: {
      id: `label-${name}`,
      label: {
        name,
        vcpus: 8,
        memory_gb: 16,
        disk_gb: 60,
        allow_boat: boat.state !== "disabled",
        allow_machines: true,
        when_nothing_fits:
          boat.state === "largest_fallback" ? "largest_boat" : "wait",
      },
      fit: {
        eligible_machines: 3,
        fitting_machines: own === "fits" ? 3 : 0,
        no_machine_fits: own === "none",
        unavailable_machines: 0,
        minimum_jobs: own === "fits" ? "2" : null,
        maximum_jobs: own === "fits" ? "2" : null,
      },
    },
    boat,
  };
}

const connectedBoat: JobOverflow = {
  connected: true,
  key_rejected: false,
  billing_required: false,
  retry_at: null,
  limit_cents: "10000",
  counted_cents: "1200",
  limit_reached: false,
  measured_complete: true,
  next_month: "2026-12-01T00:00:00Z",
  refused_sizes: [],
};

/** The live queue with every own slot in use and these waiting labels. */
function fullQueue(
  labels: WaitingLabelFit[],
  overflow: JobOverflow,
): JobsLivePage {
  const base = liveJobs("live", "admin", null);
  const count = labels.reduce((sum, fit) => sum + Number(fit.waiting_jobs), 0);
  const job = fixtureJob({ status: "queued", location: null });
  return {
    ...base,
    own_slots_total: "6",
    own_slots_used: "6",
    waiting: Array.from({ length: count }, (_, i) => ({
      ...job,
      id: `waiting-${i}`,
    })),
    waiting_count: String(count),
    waiting_labels: labels,
    overflow,
  };
}

const fitting: BoatLabelFit = {
  state: "fitting",
  size: { vcpus: 8, memory_bytes: "16000000000", disk_bytes: "120000000000" },
  size_name: "large",
  refused: false,
};

const largest: BoatLabelFit = {
  state: "largest_fallback",
  size: { vcpus: 16, memory_bytes: "64000000000", disk_bytes: "240000000000" },
  size_name: "xlarge",
  refused: false,
};

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
        "Jobs whose label fits your machines wait there for a free slot. Boat takes jobs again on 1\u00a0December.",
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
];

test("the live queue promises a free slot or a Boat start only from the confirmed facts of each waiting label", () => {
  assert.deepEqual(
    placeCases.map((item) => [
      item.name,
      liveNotices(
        fullQueue(item.labels, item.overflow),
        "/settings?workspace=w",
      ).map((notice) => [notice.key, notice.body]),
    ]),
    placeCases.map((item) => [item.name, item.notices]),
  );
});
