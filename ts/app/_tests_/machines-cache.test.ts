import assert from "node:assert/strict";
import test from "node:test";

import {
  cacheHits,
  cacheText,
  cacheUse,
  clearCacheBody,
  dockerUse,
  machineHits,
  unknownNote,
} from "../src/view/machines-cache.ts";
import { canClearCache } from "../src/view/machines-panels.ts";
import { machine } from "../scripts/fixtures-setup.ts";
import {
  cache,
  docker,
  measuredCache,
  nbsp,
  unavailable,
} from "./machines-fixtures.ts";

test("the cache shows its measured use, and only an Admin clears an online cache", () => {
  assert.deepEqual(
    [
      cacheUse(measuredCache("91", "100")),
      machineHits(measuredCache("91", "100")),
      dockerUse(docker("3")),
    ],
    ["83 of 191 GB", "91%", `3${nbsp}repositories${nbsp}· 96${nbsp}GB`],
  );
  assert.equal(machineHits(measuredCache("0", "0")), "No restores yet");
  assert.equal(
    dockerUse(docker("1")),
    `1${nbsp}repository${nbsp}· 96${nbsp}GB`,
  );
  assert.equal(
    dockerUse(docker("9007199254740993")),
    `9,007,199,254,740,993${nbsp}repositories${nbsp}· 96${nbsp}GB`,
    "a count past the safe integers of JavaScript stays exact",
  );
  assert.equal(
    dockerUse(docker("0", "0")),
    `0${nbsp}repositories${nbsp}· 0${nbsp}GB`,
    "a measured report of no Docker disks shows zero",
  );
  const reported = machine({ cache: cache("91", "100") });
  assert.equal(canClearCache(reported, true), true);
  assert.equal(canClearCache(reported, false), false);
  assert.equal(
    canClearCache(machine({ cache: cache("1", "1"), offline: true }), true),
    false,
  );
  assert.equal(canClearCache(machine({ cache: null }), true), false);
  const unmeasured = cache("0", "0", {
    machine_cache: unavailable,
    docker_disks: unavailable,
  });
  assert.equal(
    canClearCache(machine({ cache: unmeasured }), true),
    true,
    "a stored report lets an Admin clear the cache while its figures are unknown",
  );
});

test("one note names each part of the cache that the machine did not measure, and Clear cache names no unknown size", () => {
  const cases: [string, Parameters<typeof cache>[2], string | null][] = [
    [
      "machine cache",
      { machine_cache: unavailable },
      "Cache disk and cache hits show after the machine measures them.",
    ],
    [
      "Docker disks",
      { docker_disks: unavailable },
      "Docker disks show after the machine measures them.",
    ],
    [
      "both parts",
      { machine_cache: unavailable, docker_disks: unavailable },
      "Cache disk, cache hits, and Docker disks show after the machine measures them.",
    ],
    [
      "measured zeros",
      {
        machine_cache: measuredCache("0", "0"),
        docker_disks: docker("0", "0"),
      },
      null,
    ],
  ];
  for (const [name, overrides, note] of cases) {
    assert.equal(unknownNote(cache("91", "100", overrides)), note, name);
  }
  const rest =
    "on this machine. The GitHub cache keeps its entries, so the next jobs restore from there.";
  assert.equal(
    clearCacheBody(machine({ cache: cache("91", "100") })),
    `ciproxy deletes the 83${nbsp}GB of cache entries ${rest}`,
  );
  assert.equal(
    clearCacheBody(
      machine({ cache: cache("91", "100", { machine_cache: unavailable }) }),
    ),
    `ciproxy deletes the cache entries ${rest}`,
    "the confirmation names no size that the machine did not measure",
  );
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
