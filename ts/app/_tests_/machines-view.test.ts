import assert from "node:assert/strict";
import test from "node:test";

import { concurrentJobs, joinedFits } from "../src/view/machines-capacity.ts";
import {
  elsewhereText,
  machineFigures,
  machineStatus,
  slotBar,
  slotCaption,
} from "../src/view/machines.ts";
import {
  addMachineHref,
  machineHref,
  machinesRoute,
} from "../src/view/machines-routes.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { cache, nbsp, slot, summary } from "./machines-fixtures.ts";

test("a machine shows one state: offline, no slots, draining, preparing, then healthy", () => {
  const cases: [string, Parameters<typeof machine>[0], string][] = [
    ["offline and draining", { state: "offline", draining: true }, "offline"],
    ["no KVM with reported slots", { kvm: false, slots: 6 }, "no-slots"],
    ["no KVM while preparing", { state: "preparing", kvm: false }, "no-slots"],
    ["KVM without room for a slot", { slots: 0 }, "no-slots"],
    ["draining", { draining: true }, "draining"],
    [
      "draining while preparing",
      { state: "preparing", draining: true },
      "draining",
    ],
    ["getting its first job image", { state: "preparing" }, "preparing"],
    ["healthy", {}, "healthy"],
  ];
  for (const [name, overrides, expected] of cases) {
    assert.equal(machineStatus(machine(overrides)), expected, name);
  }
});

test("the slots column counts busy and warm slots, and never invents free ones", () => {
  const busy = machine({
    occupancy: [
      slot(0, "job"),
      slot(1, "job"),
      slot(2, "warm"),
      slot(3, "free"),
    ],
  });
  assert.equal(slotCaption(busy), `2${nbsp}busy${nbsp}· 1${nbsp}warm`);
  assert.equal(slotCaption(machine({ offline: true })), "No slots available");
  assert.equal(
    slotCaption(machine({ state: "preparing", occupancy: [slot(0, "free")] })),
    "Getting the job image",
  );
  assert.equal(slotCaption(machine({ kvm: false })), "No slots");
  assert.equal(
    slotCaption(machine({ slots: 6, occupancy: [] })),
    `6${nbsp}slots`,
  );
});

test("the list shows no slot bar while a machine is offline or gets its first job image", () => {
  const occupancy = [slot(1, "warm"), slot(0, "job"), slot(2, "free")];
  assert.deepEqual(slotBar(machine({ occupancy })), ["job", "warm", "free"]);
  assert.deepEqual(slotBar(machine({ draining: true, occupancy })), [
    "job",
    "warm",
    "free",
  ]);
  assert.deepEqual(slotBar(machine({ state: "offline", occupancy })), []);
  assert.deepEqual(slotBar(machine({ state: "preparing", occupancy })), []);
});

test("the figures count live slots, unique warm runners, and measured cache restores", () => {
  const machines = [
    machine({
      id: "a",
      hostname: "forge-a",
      slots: 6,
      occupancy: [
        slot(0, "job"),
        slot(1, "job", "runner-big", "job-big"),
        slot(2, "job", "runner-big", "job-big"),
        slot(3, "warm", "warm-big"),
        slot(4, "warm", "warm-big"),
        slot(5, "free"),
      ],
      cache: cache("90", "100"),
    }),
    machine({
      id: "b",
      hostname: "forge-b",
      draining: true,
      slots: 4,
      occupancy: [slot(0, "job")],
    }),
    machine({
      id: "c",
      hostname: "forge-c",
      offline: true,
      slots: 6,
      occupancy: [slot(0, "job"), slot(1, "warm")],
      cache: cache("1", "3"),
    }),
    machine({ id: "d", hostname: "forge-d", kvm: false, slots: 6 }),
    machine({
      id: "e",
      hostname: "forge-e",
      state: "preparing",
      slots: 6,
      occupancy: [0, 1, 2, 3, 4, 5].map((index) => slot(index, "free")),
    }),
    machine({
      id: "f",
      hostname: "forge-f",
      state: "preparing",
      draining: true,
      slots: 4,
      occupancy: [0, 1, 2, 3].map((index) => slot(index, "free")),
    }),
  ];
  assert.deepEqual(machineFigures(machines), {
    machines: "6",
    slots: "4 of 10",
    warm: "1",
    cacheHits: {
      kind: "share",
      value: "88%",
      excluded: ["forge-b", "forge-d", "forge-e", "forge-f"],
    },
  });
});

test("jobs elsewhere name only machines and a Boat account that take jobs now", () => {
  const offline = machine({ id: "off", offline: true });
  const healthy = machine({ id: "ok" });
  const draining = machine({ id: "drain", draining: true });
  const both = [offline, healthy];
  assert.equal(
    elsewhereText(offline, both, summary()),
    "Jobs run on your other machines and on Boat.",
  );
  assert.equal(
    elsewhereText(offline, both, summary({ enabled: false })),
    "Jobs run on your other machines.",
  );
  assert.equal(
    elsewhereText(offline, [offline, draining], summary()),
    "Jobs run on Boat.",
  );
  const preparing = machine({ id: "new", state: "preparing" });
  assert.equal(
    elsewhereText(offline, [offline, preparing], summary()),
    "Jobs run on Boat.",
    "a machine that gets its first job image takes no jobs yet",
  );
  const rejected = summary({
    account: {
      connected_at: "2026-09-15T09:00:00Z",
      name: "mokly",
      rejected_at: "2026-11-12T08:00:00Z",
    },
  });
  assert.equal(
    elsewhereText(offline, [offline], rejected),
    "Jobs wait until a machine can take them.",
  );
  const limit = summary({
    spend: {
      bounded: true,
      confirmed_cents: "20000",
      limit_reached: true,
      settled_bound_cents: "0",
      unconfirmed_cents: "0",
    },
  });
  assert.equal(
    elsewhereText(offline, [offline], limit),
    "Jobs wait until a machine can take them.",
  );
  assert.equal(
    elsewhereText(offline, [offline], null),
    "Jobs wait until a machine can take them.",
  );
});

test("the jobs of ciproxy at a time show only when the API knows them", () => {
  assert.equal(concurrentJobs(machine({ concurrent_default_jobs: "3" })), 3);
  assert.equal(
    concurrentJobs(
      machine({ capacity_known: false, concurrent_default_jobs: "3" }),
    ),
    null,
  );
  assert.equal(
    concurrentJobs(machine({ concurrent_default_jobs: null })),
    null,
  );
  assert.equal(concurrentJobs(machine({ offline: true })), null);
  assert.equal(concurrentJobs(machine({ kvm: false })), null);
  for (const draining of [false, true]) {
    const preparing = { state: "preparing" as const, draining };
    assert.equal(
      concurrentJobs(machine({ ...preparing, concurrent_default_jobs: "3" })),
      null,
      "a machine that gets its first job image runs no job yet",
    );
  }
  assert.equal(
    joinedFits(
      machine({
        state: "preparing",
        concurrent_default_jobs: "6",
        disk_limited: false,
      }),
    ),
    null,
  );
  assert.equal(
    joinedFits(machine({ concurrent_default_jobs: "6", disk_limited: false })),
    `6${nbsp}jobs of ciproxy fit at a time.`,
  );
  assert.equal(
    joinedFits(machine({ concurrent_default_jobs: "3", disk_limited: true })),
    `Its disk holds 3${nbsp}jobs of ciproxy at a time.`,
  );
  assert.equal(joinedFits(machine({ concurrent_default_jobs: null })), null);
});

test("each page of Machines has a static path, with the IDs in the query", () => {
  const id = "30000000-0000-4000-8000-0000000000aa";
  assert.equal(
    machineHref(workspaceId, id),
    `/machines?workspace=${workspaceId}&machine=${id}`,
  );
  assert.equal(
    addMachineHref(workspaceId),
    `/machines?workspace=${workspaceId}&view=add`,
  );
  assert.deepEqual(machinesRoute(undefined, id), {
    kind: "machine",
    machineId: id,
  });
  assert.deepEqual(machinesRoute("add", id), { kind: "add" });
  assert.deepEqual(machinesRoute(undefined, ["a", "b"]), { kind: "list" });
  assert.deepEqual(machinesRoute(undefined, ""), { kind: "list" });
});
