import { deepStrictEqual } from "node:assert";
import { test } from "node:test";

import { costClaimFor } from "../src/data/cost-claim.ts";
import { cheapestPlan } from "../src/data/plan-choice.ts";
import type { ListPrice } from "../src/data/price-lists.ts";

/** A price per minute of a hosted runner, for a hand-worked claim. */
function perMinute(amount: number): ListPrice {
  return { amount, item: "per minute", source: "test", checked: "test" };
}

test("cheapestPlan picks the cheapest tier with add-ons that covers a use", () => {
  const cases = [
    {
      use: "1,500 starts on 1 machine: Free covers it",
      starts: 1_500,
      machines: 1,
      expected: { tier: "free", blocks: 0, machines: 0, price: 0 },
    },
    {
      use: "2,001 starts: Free sells no add-ons, so Team",
      starts: 2_001,
      machines: 1,
      expected: { tier: "team", blocks: 0, machines: 0, price: 49 },
    },
    {
      use: "51,840 starts: 26,840 over Team round up to 3 blocks, 49 + 30",
      starts: 51_840,
      machines: 1,
      expected: { tier: "team", blocks: 3, machines: 0, price: 79 },
    },
    {
      use: "4 machines on Team: one extra machine, 49 + 20",
      starts: 25_000,
      machines: 4,
      expected: { tier: "team", blocks: 0, machines: 1, price: 69 },
    },
    {
      use: "150,000 starts on 10 machines: Team would cost 49 + 130 + 140",
      starts: 150_000,
      machines: 10,
      expected: { tier: "business", blocks: 0, machines: 0, price: 299 },
    },
  ];
  for (const { use, starts, machines, expected } of cases) {
    const choice = cheapestPlan(starts, machines);
    deepStrictEqual(
      {
        tier: choice.tier.key,
        blocks: choice.extraStartBlocks,
        machines: choice.extraMachines,
        price: choice.price,
      },
      expected,
      use,
    );
  }
});

test("costClaimFor compares the slot minutes of a month with the cheaper runner", () => {
  // 8 threads and 24 GB, less the host quarter: 3 slots by threads and 2 by
  // memory, so 2 slots. 2 slots x 30 days x 1,440 minutes = 86,400 minutes.
  // At 7 minutes a job: 12,342.9, so 12,343 starts, which Team covers. The
  // machine and Team: 56 + 49 = 105. The runners: 86,400 x 0.006 = 518.40,
  // and 86,400 x 0.004 = 345.60, the cheaper. 1 - 105 / 345.60 = 69.6%, down
  // to 69%. 345.60 - 105 = 240.60.
  const claim = costClaimFor({
    machine: "Test machine",
    hardware: { threads: 8, memoryGb: 24, diskGb: 512 },
    machinePrice: 56,
    runners: [
      { name: "B", runners: "B runners", price: perMinute(0.006) },
      { name: "A", runners: "A runners", price: perMinute(0.004) },
    ],
    averageJobMinutes: 7,
    monthDays: 30,
  });
  deepStrictEqual(
    {
      slots: claim.slots,
      jobMinutes: claim.jobMinutes,
      starts: claim.starts,
      plan: [claim.plan.tier.key, claim.plan.price],
      withCiproxy: claim.withCiproxy,
      costs: claim.alternatives.map((runner) => [runner.name, runner.cost]),
      cheaper: claim.cheaper.name,
      upToPercent: claim.upToPercent,
      saved: claim.saved,
    },
    {
      slots: 2,
      jobMinutes: 86_400,
      starts: 12_343,
      plan: ["team", 49],
      withCiproxy: 105,
      costs: [
        ["B", 518.4],
        ["A", 345.6],
      ],
      cheaper: "A",
      upToPercent: 69,
      saved: 240.6,
    },
  );
});

test("costClaimFor limits job minutes by disk while keeping hardware slots", () => {
  for (const { diskGb, expected } of [
    { diskGb: 250, expected: [6, 1, 43_200, 8_640] },
    { diskGb: 512, expected: [6, 3, 129_600, 25_920] },
    { diskGb: 1_024, expected: [6, 6, 259_200, 51_840] },
    { diskGb: 140, expected: [6, 0, 0, 0] },
  ]) {
    const claim = costClaimFor({
      machine: "Test machine",
      hardware: { threads: 16, memoryGb: 64, diskGb },
      machinePrice: 119,
      runners: [{ name: "A", runners: "A runners", price: perMinute(0.004) }],
      averageJobMinutes: 5,
      monthDays: 30,
    });
    deepStrictEqual(
      [claim.slots, claim.concurrentJobs, claim.jobMinutes, claim.starts],
      expected,
      `${diskGb} GB filesystem`,
    );
  }
});
