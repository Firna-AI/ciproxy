import assert from "node:assert/strict";
import test from "node:test";

import { headerActions, machineNotice } from "../src/view/machines-detail.ts";
import {
  cacheModel,
  canClearCache,
  costModel,
  healthModel,
} from "../src/view/machines-panels.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import { cache, nbsp, slot } from "./machines-fixtures.ts";

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
  const limited = {
    hostname: "office-mini-1",
    concurrent_default_jobs: "3",
    default_root_disk_bytes: "81000000000",
    job_disk_bytes: "279000000000",
    disk_limited: true,
  };
  const disk = machineNotice(machine(limited), now);
  assert.deepEqual(
    [disk?.key, disk?.tone, disk?.title, disk?.body],
    [
      "disk-limit",
      "warning",
      `Disk limits office-mini-1 to 3${nbsp}jobs of ciproxy at a time`,
      `With the current image, a job of ciproxy holds 81${nbsp}GB of disk. The machine has 279${nbsp}GB for jobs after the VM images and the host reserve. Add disk to run more jobs at once.`,
    ],
  );
  assert.equal(
    machineNotice(machine({ ...limited, disk_limited: false }), now),
    null,
  );
  assert.equal(
    machineNotice(machine({ ...limited, disk_limited: null }), now),
    null,
  );
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

test("health marks the values of the last contact as old while offline", () => {
  const old = healthModel(
    machine({
      offline: true,
      health_old: true,
      last_contact_at: lostAt,
      agent_version: "0.4.2",
      latest_agent_version: "0.4.3",
      agent_up_to_date: false,
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
  const lastKvm = healthModel(
    machine({ offline: true, health_old: true, kvm: false, disks: [] }),
    now,
  ).rows[1];
  assert.deepEqual(
    [lastKvm?.label, lastKvm?.value, lastKvm?.tone, lastKvm?.old],
    ["KVM", "Not available", "idle", true],
    "a failed KVM check of the last contact is old while offline",
  );
});

test("the agent claims the newest release only from a fresh report that matches it", () => {
  const agent = (overrides: Parameters<typeof machine>[0]) => {
    const row = healthModel(machine(overrides), now).rows.at(-1);
    return [row?.label, row?.value, row?.tone];
  };
  assert.deepEqual(
    agent({
      agent_version: "0.4.2",
      latest_agent_version: "0.4.2",
      agent_up_to_date: true,
    }),
    ["Agent", `0.4.2${nbsp}· up to date`, "ok"],
  );
  assert.deepEqual(
    agent({
      agent_version: "0.4.1",
      latest_agent_version: "0.4.2",
      agent_up_to_date: false,
    }),
    ["Agent", `0.4.1${nbsp}· 0.4.2${nbsp}available`, "warn"],
  );
  assert.deepEqual(
    agent({
      agent_version: "0.4.2",
      latest_agent_version: null,
      agent_up_to_date: null,
    }),
    ["Agent", "0.4.2", "idle"],
  );
  assert.deepEqual(
    agent({
      agent_version: "0.4.2",
      latest_agent_version: "0.4.2",
      agent_up_to_date: true,
      offline: true,
      health_old: true,
    }),
    ["Agent", "0.4.2", "idle"],
  );
});

test("the cache shows its reported use, and only an Admin clears an online cache", () => {
  const reported = machine({ cache: cache("91", "100") });
  const model = cacheModel(reported);
  assert.deepEqual(
    model.reported ? [model.value, model.hits, model.docker] : null,
    ["83 of 191 GB", "91%", `3${nbsp}repositories${nbsp}· 96${nbsp}GB`],
  );
  const one = cacheModel(
    machine({ cache: cache("0", "0", { docker_repositories: "1" }) }),
  );
  assert.equal(
    one.reported ? one.docker : null,
    `1${nbsp}repository${nbsp}· 96${nbsp}GB`,
  );
  const many = cacheModel(
    machine({
      cache: cache("0", "0", { docker_repositories: "9007199254740993" }),
    }),
  );
  assert.equal(
    many.reported ? many.docker : null,
    `9,007,199,254,740,993${nbsp}repositories${nbsp}· 96${nbsp}GB`,
    "a count past the safe integers of JavaScript stays exact",
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
