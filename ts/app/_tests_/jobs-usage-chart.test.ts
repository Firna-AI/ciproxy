import assert from "node:assert/strict";
import test from "node:test";

import { usageChartModel } from "../src/view/jobs-usage-chart.ts";
import { at, figures, measured, plain, step } from "./jobs-usage-support.ts";

test("the chart puts each point at the middle of its time and marks the steps, the kills, and the label size", () => {
  const usage = measured({
    job: figures(1167, "8000000000", 1),
    label_mark: { vcpus: 1, memory_bytes: "6000000000" },
    steps: [
      step(1, "Set up job", 0, 1),
      step(2, "Build", 1, 4),
      step(3, "Post Run actions/setup-node@v4", null, null, { skipped: true }),
      step(4, "Post Run actions/checkout@v4", 4, 4),
      step(5, "Complete job", 4, 6),
    ],
  });
  const chart = usageChartModel(usage, false);
  assert.deepEqual(
    {
      seconds: chart.seconds,
      lane: chart.lane,
      starts: chart.starts,
      times: chart.times,
      labelKey: plain(chart.labelKey),
      killKey: chart.killKey,
      summary: plain(chart.summary),
    },
    {
      seconds: 5,
      lane: [
        { number: 1, left: 0, width: 0.2 },
        { number: 2, left: 0.2, width: 0.6 },
        { number: 5, left: 0.8, width: 0.2 },
      ],
      starts: [1, 4],
      times: [
        { text: "0:00", share: 0 },
        { text: "0:05", share: 1 },
      ],
      labelKey: "Label size: 1 vCPU · 6 GB",
      killKey: true,
      summary:
        "CPU and memory over 5s: average CPU 1.2 vCPU of 2 vCPU, peak memory 8.0 GB of 8 GB. The label gives 1 vCPU and 6 GB.",
    },
  );
  const [cpu, memory] = chart.plots;
  assert.deepEqual(
    [cpu.title, cpu.unit, cpu.ticks, cpu.line, cpu.area, cpu.label, cpu.kills],
    [
      "CPU",
      "vCPU",
      [
        { text: "2", share: 1 },
        { text: "1", share: 0.5 },
        { text: "0", share: 0 },
      ],
      "M0,50L1,50L3,0L4.5,75L5,75",
      "M0,100L0,50L1,50L3,0L4.5,75L5,75L5,100Z",
      50,
      [],
    ],
  );
  assert.deepEqual(
    [memory.title, memory.ticks, memory.line, memory.label, memory.kills],
    [
      "Memory",
      [
        { text: "8", share: 1 },
        { text: "4", share: 0.5 },
        { text: "0", share: 0 },
      ],
      "M0,75L1,75L3,50L4.5,0L5,0",
      25,
      [{ left: 0.6, top: 0.5 }],
    ],
  );
  assert.equal(usageChartModel(measured(), false).lane, null);
});

test("the time axis spaces its ticks for the length of the job and the width of the window", () => {
  const ticks = (seconds: number, mobile: boolean) =>
    usageChartModel(
      measured({
        chart: {
          started_at: at(0),
          ended_at: at(seconds),
          point_seconds: 1,
          points: [figures(0, "0")],
        },
      }),
      mobile,
    ).times.map((tick) => tick.text);
  assert.deepEqual(ticks(372, false), [
    "0:00",
    "1:00",
    "2:00",
    "3:00",
    "4:00",
    "5:00",
    "6:00",
  ]);
  assert.deepEqual(ticks(372, true), ["0:00", "2:00", "4:00", "6:00"]);
  assert.deepEqual(ticks(114, false), [
    "0:00",
    "0:15",
    "0:30",
    "0:45",
    "1:00",
    "1:15",
    "1:30",
    "1:45",
  ]);
  assert.deepEqual(ticks(7200, false), [
    "0:00",
    "15:00",
    "30:00",
    "45:00",
    "1:00:00",
    "1:15:00",
    "1:30:00",
    "1:45:00",
    "2:00:00",
  ]);
});
