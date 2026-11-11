import assert from "node:assert/strict";
import test from "node:test";

import { slotJobHref } from "../src/view/machines-routes.ts";
import { slotsModel, type SlotsModel } from "../src/view/machines-slots.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { slotJob, slotJobIds } from "../scripts/m20-fixtures-machines.ts";
import { nbsp, slot, summary } from "./machines-fixtures.ts";

function rows(model: SlotsModel) {
  return model.kind === "slots" ? model.rows : [];
}

test("each slot row keeps its reported index, in index order", () => {
  const model = slotsModel(
    machine({
      slots: 6,
      concurrent_default_jobs: "3",
      occupancy: [slot(4, "warm"), slot(0, "job"), slot(2, "free")],
    }),
    [],
    null,
  );
  assert.deepEqual(
    rows(model).map((row) => [row.label, row.text]),
    [
      ["Slot 1", "Running a job"],
      ["Slot 3", "Free"],
      ["Slot 5", "Warm runner, ready"],
    ],
  );
  assert.equal(
    model.kind === "slots" ? model.fits : null,
    `3${nbsp}jobs of ciproxy fit at a time.`,
  );
  const one = slotsModel(
    machine({ occupancy: [slot(0, "free")], concurrent_default_jobs: "1" }),
    [],
    null,
  );
  assert.equal(
    one.kind === "slots" ? one.fits : null,
    `1${nbsp}job of ciproxy fits at a time.`,
  );
});

test("a machine without slots says where its jobs go instead", () => {
  const lab = machine({ id: "lab", kvm: false });
  const other = machine({ id: "other" });
  assert.deepEqual(slotsModel(lab, [lab, other], summary()), {
    kind: "message",
    text: "No slots. Jobs run on your other machines and on Boat.",
  });
  assert.deepEqual(slotsModel(machine({ offline: true }), [], null), {
    kind: "message",
    text: "Slots show again when the machine is back.",
  });
});

test("a busy slot names the job that the API matched, by its canonical ID", () => {
  const build = slotJob({
    id: slotJobIds.build,
    name: "build (linux-x64)",
    repository: "northstar-labs/runner",
    git_ref: "refs/heads/main",
    head_branch: "main",
    runtime_seconds: "3725.5",
  });
  const model = slotsModel(
    machine({
      occupancy: [
        { ...slot(0, "job", "runner-a", "report-a"), job: slotJob() },
        { ...slot(1, "job", "runner-b", "report-b"), job: build },
        { ...slot(2, "job", "runner-b", "report-b"), job: build },
        { ...slot(3, "job", "runner-c", "report-c"), job: null },
        slot(4, "free"),
      ],
    }),
    [],
    null,
  );
  assert.deepEqual(
    rows(model).map((row) => [row.text, row.job]),
    [
      [
        "test (rust)",
        {
          id: slotJobIds.test,
          name: "test (rust)",
          context: "control #1542",
          duration: `4m${nbsp}12s`,
        },
      ],
      [
        "build (linux-x64)",
        {
          id: slotJobIds.build,
          name: "build (linux-x64)",
          context: "runner main",
          duration: `1h${nbsp}02m`,
        },
      ],
      [
        "build (linux-x64)",
        {
          id: slotJobIds.build,
          name: "build (linux-x64)",
          context: "runner main",
          duration: `1h${nbsp}02m`,
        },
      ],
      ["Running a job", null],
      ["Free", null],
    ],
  );
  assert.equal(
    slotJobHref("workspace one", slotJobIds.test),
    `/job?workspace=workspace+one&job=${slotJobIds.test}`,
  );
});

test("a disk that serves fewer jobs than the slots marks the free slots beyond it", () => {
  const limited = {
    slots: 6,
    default_jobs_by_slots: "6",
    concurrent_default_jobs: "3",
    disk_limited: true,
    occupancy: [
      slot(0, "warm"),
      slot(1, "warm"),
      ...[2, 3, 4, 5].map((index) => slot(index, "free")),
    ],
  };
  const texts = (overrides: Parameters<typeof machine>[0]) =>
    rows(slotsModel(machine({ ...limited, ...overrides }), [], null)).map(
      (row) => row.text,
    );
  const noDisk = "Free, no disk for a job";
  assert.deepEqual(texts({}), [
    "Warm runner, ready",
    "Warm runner, ready",
    "Free",
    noDisk,
    noDisk,
    noDisk,
  ]);
  assert.deepEqual(
    texts({
      occupancy: [
        slot(0, "free"),
        slot(1, "warm"),
        slot(2, "free"),
        slot(3, "warm"),
        slot(4, "free"),
        slot(5, "free"),
      ],
    }),
    [
      "Free",
      "Warm runner, ready",
      noDisk,
      "Warm runner, ready",
      noDisk,
      noDisk,
    ],
  );
  const plain = ["Warm runner, ready", "Warm runner, ready"].concat(
    Array<string>(4).fill("Free"),
  );
  assert.deepEqual(texts({ disk_limited: false }), plain);
  assert.deepEqual(texts({ disk_limited: null }), plain);
  assert.deepEqual(texts({ default_jobs_by_slots: "3" }), plain);
});
