import assert from "node:assert/strict";
import test from "node:test";

import {
  headerActions,
  machineNotice,
  slotsModel,
} from "../src/view/machines-detail.ts";
import {
  cacheModel,
  canClearCache,
  costModel,
  healthModel,
} from "../src/view/machines-panels.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { cache, nbsp, slot, summary } from "./machines-fixtures.ts";

const now = Date.parse("2026-11-18T14:32:00Z");
const lostAt = "2026-11-18T14:18:00Z";

function labels(overrides: Parameters<typeof machine>[0]): string[] {
  return headerActions(machine(overrides)).map((action) => action.label);
}

test("an Admin gets the actions of each machine state", () => {
  assert.deepEqual(labels({}), ["Drain", "Remove"]);
  assert.deepEqual(labels({ draining: true }), ["Stop draining"]);
  assert.deepEqual(labels({ offline: true, draining: true }), [
    "Remove machine",
  ]);
  assert.deepEqual(labels({ kvm: false }), ["Remove"]);
  assert.deepEqual(labels({ kvm: false, draining: true }), [
    "Stop draining",
    "Remove",
  ]);
  const commands = headerActions(machine({})).map((action) => action.command);
  assert.deepEqual(commands, ["drain", "remove"]);
});

test("the notice says why a machine takes no jobs, with the fix where one exists", () => {
  const offline = machineNotice(
    machine({ hostname: "forge-03", offline: true, last_contact_at: lostAt }),
    now,
  );
  assert.equal(offline?.title, "forge-03 is offline");
  assert.equal(
    offline?.body,
    `ciproxy lost contact with it 14${nbsp}minutes ago. Check that the machine is on and can reach ciproxy.dev.`,
  );
  const noKvm = machineNotice(machine({ hostname: "lab", kvm: false }), now);
  assert.deepEqual(
    [noKvm?.key, noKvm?.title, noKvm?.fix],
    ["no-kvm", "lab cannot start VMs", true],
  );
  const noRoom = machineNotice(machine({ hostname: "tiny", slots: 0 }), now);
  assert.deepEqual(
    [noRoom?.key, noRoom?.title, noRoom?.fix],
    ["no-slots", "tiny has no slots", false],
  );
  assert.equal(machineNotice(machine({}), now), null);
});

test("a drain counts each running job once, also a job of several slots", () => {
  const draining = machine({
    draining: true,
    occupancy: [
      slot(0, "job", "runner-a", "job-a"),
      slot(1, "job", "runner-a", "job-a"),
      slot(2, "job", "runner-b", "job-b"),
      slot(3, "free"),
    ],
  });
  assert.equal(
    machineNotice(draining, now)?.body,
    `2${nbsp}jobs are finishing. The machine takes no new jobs, and it keeps no warm runners.`,
  );
  assert.equal(
    machineNotice(machine({ draining: true }), now)?.body,
    "The machine takes no new jobs, and it keeps no warm runners.",
  );
});

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
    model.kind === "slots"
      ? model.rows.map((row) => [row.label, row.text])
      : null,
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

test("health marks the values of the last contact as old while offline", () => {
  const old = healthModel(
    machine({
      offline: true,
      health_old: true,
      last_contact_at: lostAt,
      agent_version: "0.4.2",
      disks: [
        {
          path: "/",
          capacity_bytes: "512000000000",
          free_bytes: "312000000000",
        },
        {
          path: "/data",
          capacity_bytes: "512000000000",
          free_bytes: "100400000000",
        },
      ],
    }),
    now,
  );
  assert.deepEqual(
    old.rows.map((row) => [row.label, row.value, row.tone]),
    [
      ["Last contact", "14m ago", "err"],
      ["KVM", "Ready", "idle"],
      ["Disk /", `312${nbsp}GB free`, "idle"],
      ["Disk /data", `100${nbsp}GB free`, "idle"],
      ["Agent", "0.4.2", "idle"],
    ],
  );
  assert.equal(
    old.note,
    `KVM, disk, and agent show the values of the last contact, 14${nbsp}minutes ago.`,
  );
  const missing = healthModel(machine({ kvm: false, disks: [] }), now);
  assert.deepEqual(
    missing.rows.slice(1, 3).map((row) => [row.label, row.value, row.tone]),
    [
      ["KVM", "Not available", "err"],
      ["Disk", "No report yet", "idle"],
    ],
  );
  assert.equal(missing.note, null);
});

test("the cache shows its reported use, and only an Admin clears an online cache", () => {
  const reported = machine({ cache: cache("91", "100") });
  const model = cacheModel(reported);
  assert.deepEqual(
    model.reported ? [model.value, model.hits, model.docker] : null,
    ["83 of 191 GB", "91%", `96${nbsp}GB`],
  );
  assert.equal(canClearCache(reported, true), true);
  assert.equal(canClearCache(reported, false), false);
  assert.equal(
    canClearCache(machine({ cache: cache("1", "1"), offline: true }), true),
    false,
  );
  assert.equal(canClearCache(machine({ cache: null }), true), false);
  assert.deepEqual(cacheModel(machine({ cache: null })), {
    reported: false,
    text: "Cache use shows after the machine reports it.",
  });
});

test("a known zero cost stays apart from a cost that nobody entered", () => {
  assert.deepEqual(costModel(machine({ monthly_cost_cents: null })), {
    value: "No monthly cost yet",
    caption: "The savings report needs the monthly cost of each machine.",
    input: "",
  });
  assert.deepEqual(costModel(machine({ monthly_cost_cents: "0" })), {
    value: `$0${nbsp}a${nbsp}month`,
    caption: "The savings report uses this cost.",
    input: "0",
  });
  assert.equal(
    costModel(machine({ monthly_cost_cents: "5340" })).input,
    "53.40",
  );
});
