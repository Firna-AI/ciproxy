import assert from "node:assert/strict";
import test from "node:test";

import type { MachineView } from "../src/api/schema.ts";
import { unknownNote } from "../src/view/machines-cache.ts";
import { connectedFacts, macJobsLine } from "../src/view/machines-capacity.ts";
import { machineNotice, machineSubtitle } from "../src/view/machines-detail.ts";
import { healthModel } from "../src/view/machines-panels.ts";
import { slotsModel } from "../src/view/machines-slots.ts";
import { hardwareText } from "../src/view/machines.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { nbsp, slot } from "./machines-fixtures.ts";

const now = Date.parse("2026-11-06T12:00:30Z");

/** A Mac Studio with 16 cores and 64 GB: 6 slots, at most 2 VMs at once. */
function mac(overrides: Partial<MachineView> = {}): MachineView {
  return machine({
    hostname: "mac-studio-1",
    platform: "macos_arm64",
    processor: "Apple M4 Max",
    threads: 16,
    vm_limit: 2,
    concurrent_default_jobs: "2",
    default_jobs_by_slots: "6",
    ...overrides,
  });
}

test("a machine names its platform, and a Mac names its cores", () => {
  assert.equal(
    hardwareText(machine()),
    `Linux${nbsp}· 12${nbsp}threads${nbsp}· 64${nbsp}GB`,
  );
  assert.equal(
    machineSubtitle(mac({ disks: [] })),
    `macOS${nbsp}· Apple M4 Max${nbsp}· 16${nbsp}cores${nbsp}· 64${nbsp}GB`,
  );
});

test("the free slots of a Mac at its limit of VMs say so", () => {
  const model = slotsModel(
    mac({
      occupancy: [
        slot(0, "job"),
        slot(1, "warm"),
        slot(2, "free"),
        slot(3, "free"),
        slot(4, "free"),
        slot(5, "free"),
      ],
    }),
    [],
    null,
  );
  assert.equal(model.kind, "slots");
  if (model.kind !== "slots") return;
  assert.deepEqual(model.rows.map((row) => row.text).slice(1, 3), [
    "Warm runner, ready",
    "Free, the Mac runs its 2 VMs",
  ]);
  assert.equal(
    model.caption,
    `6${nbsp}slots of 2${nbsp}vCPU and 8${nbsp}GB, and at most 2 VMs at once. A larger VM size takes several slots.`,
  );
  assert.equal(model.fits, `2${nbsp}jobs of ciproxy-macos fit at a time.`);
});

test("a free slot of a Mac below its limit of VMs is plain", () => {
  const model = slotsModel(
    mac({ occupancy: [slot(0, "job"), slot(1, "free"), slot(2, "free")] }),
    [],
    null,
  );
  assert.deepEqual(
    model.kind === "slots" ? model.rows.map((row) => row.text) : [],
    ["Running a job", "Free", "Free"],
  );
});

test("a Mac with fewer slots than its limit of VMs says why", () => {
  const notice = machineNotice(
    mac({ hostname: "mac-mini-1", slots: 1, concurrent_default_jobs: "1" }),
    [],
    null,
    now,
  );
  assert.deepEqual(
    [notice?.key, notice?.tone, notice?.title],
    ["mac-slots", "info", `mac-mini-1 runs 1${nbsp}job at a time`],
  );
  assert.equal(machineNotice(mac(), [], null, now), null);
});

test("a Mac that cannot start VMs names the macOS fix", () => {
  const notice = machineNotice(
    mac({ virtualization: "operating_system_too_old", kvm: false }),
    [],
    null,
    now,
  );
  assert.equal(notice?.key, "no-virtualization");
  assert.equal(notice?.fix, true);
  assert.match(notice?.body ?? "", /Update macOS/);
  const health = healthModel(
    mac({ virtualization: "framework_unsupported", kvm: false }),
    now,
  );
  assert.deepEqual(health.rows.map((row) => [row.label, row.value])[1], [
    "Virtualization",
    "Not available",
  ]);
});

test("a Mac with no slots says that macOS jobs wait for a Mac", () => {
  const model = slotsModel(
    mac({ virtualization: "entitlement_missing", kvm: false, slots: 0 }),
    [],
    null,
  );
  assert.equal(
    model.kind === "message" ? model.text : null,
    "No slots. Jobs of macOS labels wait until a Mac has a free slot.",
  );
});

test("a Mac that joined names its virtualization and its macOS jobs", () => {
  assert.equal(
    connectedFacts(mac()),
    `16${nbsp}cores${nbsp}· 64${nbsp}GB${nbsp}· Virtualization${nbsp}ready${nbsp}· 6${nbsp}slots`,
  );
  assert.equal(
    macJobsLine(mac()),
    "It runs the jobs of macOS labels, at most 2 at a time.",
  );
  assert.equal(macJobsLine(machine()), null);
});

test("the cache note of a Mac never names Docker disks", () => {
  const cache = {
    machine_cache: {
      capacity_bytes: "1",
      hits_last_7_days: "0",
      restores_last_7_days: "0",
      status: "measured",
      used_bytes: "0",
    },
    docker_disks: { status: "unavailable" },
  } as const;
  assert.equal(
    unknownNote(cache),
    "Docker disks show after the machine measures them.",
  );
  assert.equal(unknownNote(cache, false), null);
});
