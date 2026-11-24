import assert from "node:assert/strict";
import test from "node:test";

import { usageChartModel } from "../src/view/jobs-usage-chart.ts";
import type { TimeAxis } from "../src/view/jobs-usage-times.ts";
import { at, figures, measured, plain, step } from "./jobs-usage-support.ts";

/**
 * A time axis of `width` pixels in the panel. Left of the axis, the panel
 * shows the 30 px column of axis labels, a 10 px gap, and its padding; right
 * of it, its padding: 16 px on a wider window, 14 px on a phone.
 */
const wide = (width: number): TimeAxis => ({ width, left: 56, right: 16 });
const phone = (width: number): TimeAxis => ({ width, left: 54, right: 14 });

/** The labels of the time axis of a chart of `seconds`. */
function times(seconds: number, mobile: boolean, axis: TimeAxis) {
  return usageChartModel(
    measured({
      chart: {
        started_at: at(0),
        ended_at: at(seconds),
        point_seconds: 1,
        points: [figures(0, "0")],
      },
    }),
    mobile,
    axis,
  ).times;
}

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
  const chart = usageChartModel(usage, false, wide(1000));
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
        { text: "0:00", share: 0, align: "center" },
        { text: "0:01", share: 0.2, align: "center" },
        { text: "0:02", share: 0.4, align: "center" },
        { text: "0:03", share: 0.6, align: "center" },
        { text: "0:04", share: 0.8, align: "center" },
        { text: "0:05", share: 1, align: "center" },
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
  assert.equal(usageChartModel(measured(), false, wide(1000)).lane, null);
});

test("the time axis spaces its ticks for the length of the job and the width of the window", () => {
  const ticks = (seconds: number, mobile: boolean, axis: TimeAxis) =>
    times(seconds, mobile, axis).map((tick) => tick.text);
  assert.deepEqual(ticks(372, false, wide(1000)), [
    "0:00",
    "1:00",
    "2:00",
    "3:00",
    "4:00",
    "5:00",
    "6:00",
  ]);
  assert.deepEqual(ticks(372, true, phone(300)), [
    "0:00",
    "2:00",
    "4:00",
    "6:00",
  ]);
  assert.deepEqual(ticks(114, false, wide(1000)), [
    "0:00",
    "0:15",
    "0:30",
    "0:45",
    "1:00",
    "1:15",
    "1:30",
    "1:45",
  ]);
  assert.deepEqual(ticks(7200, false, wide(1000)), [
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
  // On a narrow axis, as at 640 px, 2:00:00 at the panel edge would come
  // within 4.2 px of 1:45:00.
  assert.deepEqual(ticks(7200, false, wide(460)), [
    "0:00",
    "30:00",
    "1:00:00",
    "1:30:00",
    "2:00:00",
  ]);
});

test("the time axis names a time after the start and keeps each label where the panel shows it", () => {
  assert.deepEqual(times(3, false, wide(1000)), [
    { text: "0:00", share: 0, align: "center" },
    { text: "0:01", share: 1 / 3, align: "center" },
    { text: "0:02", share: 2 / 3, align: "center" },
    { text: "0:03", share: 1, align: "center" },
  ]);
  // Centred, 1:00:00 would pass the 14 px padding by 9.1 px.
  assert.deepEqual(times(3600, true, phone(300)), [
    { text: "0:00", share: 0, align: "center" },
    { text: "15:00", share: 0.25, align: "center" },
    { text: "30:00", share: 0.5, align: "center" },
    { text: "45:00", share: 0.75, align: "center" },
    { text: "1:00:00", share: 1, align: "end" },
  ]);
  // 6:00 lies 9.7 px from the end of the axis: centred, it ends 3.5 px into
  // the padding.
  assert.deepEqual(
    times(372, true, phone(300)).map((tick) => tick.align),
    ["center", "center", "center", "center"],
  );
  // With no room past the axis, the first label starts at the left end of the
  // axis, and the last label ends at its right end.
  assert.deepEqual(
    times(3, false, { width: 1000, left: 0, right: 0 }).map(
      (tick) => tick.align,
    ),
    ["start", "center", "center", "end"],
  );
});
