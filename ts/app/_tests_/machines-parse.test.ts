import assert from "node:assert/strict";
import test from "node:test";

import { parseMachines } from "../src/api/parse-machines.ts";
import { fixtureMachines } from "../scripts/m20-fixtures-machines.ts";

test("the machines reader keeps each fact, a null apart from zero, and the matched job of a slot", () => {
  const machines = fixtureMachines();
  assert.deepEqual(
    parseMachines(JSON.parse(JSON.stringify(machines))),
    machines,
  );
  const [primary] = machines;
  assert.ok(primary);
  const wire = JSON.parse(JSON.stringify(primary));
  delete wire.disk_limited;
  delete wire.latest_agent_version;
  delete wire.occupancy[1].job;
  const read = parseMachines([wire])?.[0];
  assert.deepEqual(
    [read?.disk_limited, read?.latest_agent_version, read?.occupancy[1]?.job],
    [null, null, null],
  );
  assert.deepEqual(read?.cache?.docker_disks, {
    repositories: "3",
    status: "measured",
    used_bytes: "18000000000",
  });
  for (const value of [3, "three"]) {
    const docker = { ...wire.cache.docker_disks, repositories: value };
    const cache = { ...wire.cache, docker_disks: docker };
    assert.equal(parseMachines([{ ...wire, cache }]), undefined);
  }
  const job = { ...wire.occupancy[0].job, runtime_seconds: "-4" };
  const occupancy = [{ ...wire.occupancy[0], job }];
  assert.equal(parseMachines([{ ...wire, occupancy }]), undefined);
});

test("the machines reader keeps each part of a report measured or unavailable, and no report as null", () => {
  const [primary] = fixtureMachines();
  assert.ok(primary);
  const wire = JSON.parse(JSON.stringify(primary));
  const read = (cache: unknown) =>
    parseMachines([{ ...wire, cache }])?.[0]?.cache;
  assert.deepEqual(
    read({
      machine_cache: { status: "unavailable", used_bytes: "5" },
      docker_disks: { status: "measured", repositories: "0", used_bytes: "0" },
    }),
    {
      machine_cache: { status: "unavailable" },
      docker_disks: { repositories: "0", status: "measured", used_bytes: "0" },
    },
    "an unavailable part keeps no figure, and a measured zero stays measured",
  );
  assert.deepEqual(
    read({ ...wire.cache, docker_disks: { status: "unavailable" } }),
    { ...wire.cache, docker_disks: { status: "unavailable" } },
    "the parts are apart: the machine cache keeps its figures",
  );
  assert.equal(read(null), null, "a null cache stays the state of no report");
  const measured = wire.cache.machine_cache;
  const missing = { ...measured };
  delete missing.used_bytes;
  for (const [name, machineCache] of [
    ["an unknown status", { ...measured, status: "estimated" }],
    ["no status", { used_bytes: measured.used_bytes }],
    ["a measured part without a figure", missing],
  ] as const) {
    const cache = { ...wire.cache, machine_cache: machineCache };
    assert.equal(parseMachines([{ ...wire, cache }]), undefined, name);
  }
});
