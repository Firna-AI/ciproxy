import assert from "node:assert/strict";
import test from "node:test";

import type { JobsLivePage } from "../src/api/jobs-types.ts";
import {
  jobDetailModel,
  noRunnerReason,
  unavailableReason,
} from "../src/view/jobs-detail.ts";
import { filtersOf, historyQuery, jobsHref } from "../src/view/jobs-links.ts";
import { liveModel } from "../src/view/jobs-live.ts";
import { liveNotices } from "../src/view/jobs-notices.ts";
import { fixtureJob } from "../scripts/fixtures-jobs.ts";
import { liveJobs } from "../scripts/fixtures-jobs-live.ts";

const unknownRequest = {
  name: "ciproxy",
  vcpus: null,
  memory_gb: null,
  disk_gb: null,
};

/** The words of a value. Formats keep a value on one line with no-break spaces. */
function plain(value: string | null): string | null {
  return value === null ? null : value.replaceAll("\u00a0", " ");
}

function rows(model: ReturnType<typeof jobDetailModel>) {
  return model.rows.map((row) => [
    row.label,
    plain(row.value),
    plain(row.caption),
  ]);
}

test("an executed job without its saved request shows it as unknown and names the VM that ran it", () => {
  const vm = {
    vcpus: 2,
    memory_bytes: "8000000000",
    disk_bytes: "81000000000",
  };
  const own = jobDetailModel(
    fixtureJob({ label: unknownRequest, actual_size: vm }),
  );
  assert.deepEqual(rows(own).slice(0, 2), [
    ["Label", "ciproxy", "Requested size unknown"],
    ["VM size", "2 vCPU · 8 GB · 81 GB VM disk", null],
  ]);
  const boat = jobDetailModel(
    fixtureJob({
      label: unknownRequest,
      location: { kind: "boat", size: "large" },
      actual_size: {
        vcpus: 8,
        memory_bytes: "16000000000",
        disk_bytes: "111000000000",
      },
    }),
  );
  assert.deepEqual(rows(boat).slice(0, 2), [
    ["Label", "ciproxy", "Requested size unknown"],
    [
      "VM size",
      "Boat large · 8 vCPU · 16 GB · 111 GB disk",
      "Boat bills this size.",
    ],
  ]);
  const untold = jobDetailModel(
    fixtureJob({ label: unknownRequest, actual_size: null }),
  );
  assert.deepEqual(rows(untold)[1], ["VM size", "Unknown", null]);
});

test("a finished job that no runner took is no start and names no VM, wait, or cache", () => {
  const model = jobDetailModel(
    fixtureJob({
      label: unknownRequest,
      location: null,
      actual_size: null,
      counts_as_start: false,
      result: "cancelled",
      started_at: null,
      waited_seconds: null,
      runtime_seconds: null,
      cache_source: null,
      cache_counts: null,
    }),
  );
  assert.deepEqual(
    [model.placeTitle, model.reason, model.look.label, rows(model)],
    [
      "No runner",
      noRunnerReason,
      "Cancelled",
      [["Label", "ciproxy", "Requested size unknown"]],
    ],
  );
});

test("job facts that the API leaves out stay unknown, apart from zero", () => {
  const unknown = jobDetailModel(
    fixtureJob({
      completed_at: null,
      runtime_seconds: null,
      routing_reason: null,
      cache_source: null,
      cache_counts: null,
    }),
  );
  assert.deepEqual(
    [unknown.reason, rows(unknown).slice(3)],
    [
      unavailableReason,
      [
        ["Duration", "Unknown", null],
        ["Cache", "Unknown", null],
        ["Starts", "Counts as 1 start", null],
      ],
    ],
  );
  const none = jobDetailModel(
    fixtureJob({
      cache_source: null,
      cache_counts: { restored_entries: "0", restored_bytes: "0" },
    }),
  );
  assert.deepEqual(rows(none)[4], ["Cache", "No entries restored", null]);
});

test("live figures keep unknown slots and a hidden waiting count apart from zero", () => {
  const known = liveModel(liveJobs("live", "admin", null), 0);
  const unknown = liveModel(
    {
      ...liveJobs("live", "admin", null),
      own_slots_total: null,
      own_slots_used: "4",
      waiting_count: null,
    },
    0,
  );
  const labels = (figures: typeof known.figures) =>
    figures.map((figure) => [figure.label, figure.value]);
  assert.deepEqual(
    labels(known.figures).map(([label]) => label),
    ["Running", "Waiting", "Own slots in use", "On Boat"],
  );
  assert.deepEqual(labels(unknown.figures).slice(1, 2), [
    ["Own slots in use", "Unknown"],
  ]);
});

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

test("History filters round-trip through the address and ask the API from the UTC date", () => {
  const filters = filtersOf({
    tab: "history",
    search: " e2e ",
    place: "boat",
    days: "7",
  });
  assert.equal(
    jobsHref("w 1", "history", filters),
    "/jobs?workspace=w%201&tab=history&search=e2e&place=boat&days=7",
  );
  const evening = Date.parse("2026-11-07T23:30:00-08:00");
  assert.deepEqual(historyQuery(filters, evening), {
    from: "2026-11-02",
    search: "e2e",
    place: "boat",
  });
  assert.deepEqual(filtersOf({ place: "own", days: "12" }), {
    search: "",
    place: null,
    days: 1,
  });
});
