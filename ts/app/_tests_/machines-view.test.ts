import assert from "node:assert/strict";
import test from "node:test";

import { cacheHits, cacheText } from "../src/view/machines-cache.ts";
import { concurrentJobs, joinedFits } from "../src/view/machines-capacity.ts";
import {
  elsewhereText,
  machineFigures,
  machineStatus,
  slotCaption,
} from "../src/view/machines.ts";
import {
  addMachineHref,
  machineHref,
  machinesRoute,
} from "../src/view/machines-routes.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import {
  cache,
  docker,
  measuredCache,
  nbsp,
  slot,
  summary,
  unavailable,
} from "./machines-fixtures.ts";

test("a machine shows one state: offline, then no slots, then draining", () => {
  const cases: [string, Parameters<typeof machine>[0], string][] = [
    ["offline and draining", { offline: true, draining: true }, "offline"],
    ["no KVM with reported slots", { kvm: false, slots: 6 }, "no-slots"],
    ["KVM without room for a slot", { slots: 0 }, "no-slots"],
    ["draining", { draining: true }, "draining"],
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
  assert.equal(slotCaption(machine({ kvm: false })), "No slots");
  assert.equal(
    slotCaption(machine({ slots: 6, occupancy: [] })),
    `6${nbsp}slots`,
  );
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
  ];
  assert.deepEqual(machineFigures(machines), {
    machines: "4",
    slots: "4 of 10",
    warm: "1",
    cacheHits: {
      kind: "share",
      value: "88%",
      excluded: ["forge-b", "forge-d"],
    },
  });
});

test("the cache hits cover the measured machines, name the others, and say Unknown without one", () => {
  const measured = machine({
    hostname: "forge-01",
    cache: cache("90", "100", { docker_disks: unavailable }),
  });
  const unmeasured = machine({
    hostname: "forge-02",
    cache: cache("0", "0", { machine_cache: unavailable }),
  });
  const unreported = machine({ hostname: "forge-03", cache: null });
  assert.deepEqual(cacheHits([measured, unmeasured, unreported]), {
    kind: "share",
    value: "90%",
    excluded: ["forge-02", "forge-03"],
  });
  assert.deepEqual(cacheHits([unmeasured, unreported]), { kind: "unknown" });
  const quiet = machine({
    hostname: "forge-04",
    cache: cache("0", "0", { docker_disks: docker("0", "0") }),
  });
  assert.deepEqual(
    cacheHits([quiet, unmeasured]),
    { kind: "share", value: "No restores yet", excluded: ["forge-02"] },
    "measured machines with no restores keep No restores yet",
  );
});

test("the cache column shows the measured use, Unknown, or No report yet", () => {
  const cases: [string, Parameters<typeof machine>[0], string][] = [
    ["measured", { cache: cache("1", "2") }, "83 of 191 GB"],
    [
      "measured cache beside unknown Docker disks",
      { cache: cache("1", "2", { docker_disks: unavailable }) },
      "83 of 191 GB",
    ],
    [
      "measured zero",
      {
        cache: cache("0", "0", {
          machine_cache: { ...measuredCache("0", "0"), used_bytes: "0" },
        }),
      },
      "0 of 191 GB",
    ],
    [
      "unmeasured",
      { cache: cache("1", "2", { machine_cache: unavailable }) },
      "Unknown",
    ],
    ["no report", { cache: null }, "No report yet"],
  ];
  for (const [name, overrides, expected] of cases) {
    assert.equal(cacheText(machine(overrides)), expected, name);
  }
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
