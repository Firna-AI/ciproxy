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
  assert.equal(read?.cache?.docker_repositories, "3");
  for (const [field, value] of [
    ["docker_repositories", 3],
    ["docker_repositories", "three"],
  ] as const) {
    const cache = { ...wire.cache, [field]: value };
    assert.equal(parseMachines([{ ...wire, cache }]), undefined);
  }
  const job = { ...wire.occupancy[0].job, runtime_seconds: "-4" };
  const occupancy = [{ ...wire.occupancy[0], job }];
  assert.equal(parseMachines([{ ...wire, occupancy }]), undefined);
});
