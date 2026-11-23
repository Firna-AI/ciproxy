import assert from "node:assert/strict";
import test from "node:test";

import { createApiClient } from "../src/api/client.ts";
import { parseJobDetail } from "../src/api/jobs-parse.ts";
import { createJobsEndpoints } from "../src/api/jobs.ts";
import { parseSavings } from "../src/api/savings-parse.ts";
import { createSavingsEndpoints } from "../src/api/savings.ts";
import { createBuildGate } from "../src/state/build-gate.ts";
import { fixtureJob } from "../scripts/fixtures-jobs.ts";
import { savingsReport } from "../scripts/fixtures-savings.ts";
import { response, transportQueue } from "./support.ts";

test("Jobs and Savings requests use the generated names, encoded IDs, and no empty filter", async () => {
  const queue = transportQueue(
    response({}),
    response({}),
    response({}),
    response({}),
  );
  const api = createApiClient({
    ...queue,
    csrf: () => null,
    gate: createBuildGate(),
  });
  const jobs = createJobsEndpoints(api);
  await jobs.history("opaque / id", {
    search: "",
    place: "boat",
    from: "2026-11-01",
  });
  await jobs.history("opaque / id", {
    search: "e2e tests",
    from: "2026-11-07",
  });
  await jobs.detail("opaque / id", "job/1");
  await createSavingsEndpoints(api).report("opaque / id", "this_month");
  assert.deepEqual(
    queue.calls.map(({ url, init }) => [init.method, url]),
    [
      [
        "GET",
        "/api/workspaces/opaque%20%2F%20id/jobs/history?place=boat&from=2026-11-01",
      ],
      [
        "GET",
        "/api/workspaces/opaque%20%2F%20id/jobs/history?search=e2e+tests&from=2026-11-07",
      ],
      ["GET", "/api/workspaces/opaque%20%2F%20id/jobs/job%2F1"],
      ["GET", "/api/workspaces/opaque%20%2F%20id/savings?range=this_month"],
    ],
  );
});

test("page readers keep null request sizes, exact fractions, and the banner payload", () => {
  const banner = {
    kind: "payment_failed",
    action: "update_card",
    failed_at: "2026-11-05T09:00:00Z",
    next_retry_at: null,
  } as const;
  const job = fixtureJob({
    label: { name: "ciproxy", vcpus: null, memory_gb: null, disk_gb: null },
    runtime_seconds: "372.5",
    waited_seconds: "0.25",
  });
  const detail = parseJobDetail({ job, banner, usage: { state: "running" } });
  assert.deepEqual(
    [
      detail?.job.label,
      detail?.job.runtime_seconds,
      detail?.job.waited_seconds,
      detail?.banner,
    ],
    [
      { name: "ciproxy", vcpus: null, memory_gb: null, disk_gb: null },
      "372.5",
      "0.25",
      banner,
    ],
  );
  const report = savingsReport("report", "last30_days", true, banner);
  const machines = { starts: "17760", minutes: "80604.25" };
  const parsed = parseSavings({ ...report, machines });
  assert.deepEqual([parsed?.machines, parsed?.banner], [machines, banner]);
  assert.equal(
    parseJobDetail({
      job: { ...job, runtime_seconds: "1e3" },
      banner: null,
      usage: { state: "running" },
    }),
    undefined,
  );
  assert.equal(parseSavings({ ...report, from: "2026-02-30" }), undefined);
});

test("the job page reader keeps each usage state, each null apart from zero, and refuses a wrong usage", () => {
  const job = fixtureJob();
  const step = {
    number: 1,
    name: "Set up job",
    script: false,
    skipped: false,
    started_at: "2026-11-06T11:50:00Z",
    ended_at: "2026-11-06T11:50:02.500Z",
    duration_seconds: "2.5",
    figures: {
      average_cpu_milli_vcpu: 0,
      peak_memory_bytes: "0",
      oom_kills: 0,
    },
  };
  const measured = {
    state: "measured",
    job: {
      average_cpu_milli_vcpu: 1563,
      peak_memory_bytes: "6082000000",
      oom_kills: 1,
    },
    vm: { vcpus: 8, memory_bytes: "16000000000" },
    label_mark: { vcpus: 2, memory_bytes: null },
    chart: {
      started_at: "2026-11-06T11:50:00Z",
      ended_at: "2026-11-06T11:50:03Z",
      point_seconds: 2,
      points: [
        { average_cpu_milli_vcpu: 0, peak_memory_bytes: "0", oom_kills: 0 },
        {
          average_cpu_milli_vcpu: 8000,
          peak_memory_bytes: "16000000000",
          oom_kills: 1,
        },
      ],
    },
    steps: [
      step,
      {
        ...step,
        number: 2,
        name: "Post Run actions/checkout@v4",
        skipped: true,
        started_at: null,
        ended_at: null,
        duration_seconds: null,
        figures: null,
      },
    ],
  };
  const macos = {
    ...measured,
    job: { ...measured.job, oom_kills: null },
    label_mark: null,
  };
  const read = (usage: unknown) =>
    parseJobDetail(JSON.parse(JSON.stringify({ job, banner: null, usage })))
      ?.usage;
  for (const usage of [
    measured,
    macos,
    { state: "running" },
    { state: "unavailable" },
  ]) {
    assert.deepEqual(read(usage), usage);
  }
  const withoutStart: Record<string, unknown> = { ...step };
  delete withoutStart.started_at;
  const withoutVm: Record<string, unknown> = { ...measured };
  delete withoutVm.vm;
  const chart = measured.chart;
  for (const [name, usage] of [
    ["no usage", undefined],
    ["an unknown state", { state: "partial" }],
    ["a measured job without its VM", withoutVm],
    ["a VM without a vCPU", { ...measured, vm: { ...measured.vm, vcpus: 0 } }],
    [
      "a negative CPU",
      { ...measured, job: { ...measured.job, average_cpu_milli_vcpu: -1 } },
    ],
    [
      "a part of a thousandth of a vCPU",
      { ...measured, job: { ...measured.job, average_cpu_milli_vcpu: 1.5 } },
    ],
    [
      "bytes in exponent form",
      { ...measured, job: { ...measured.job, peak_memory_bytes: "6.1e9" } },
    ],
    [
      "a kill count as text",
      { ...measured, job: { ...measured.job, oom_kills: "1" } },
    ],
    ["a step without its start", { ...measured, steps: [withoutStart] }],
    [
      "a point of no length",
      { ...measured, chart: { ...chart, point_seconds: 0 } },
    ],
    [
      "a chart without points",
      { ...measured, chart: { ...chart, points: [] } },
    ],
    [
      "a chart that ends before it starts",
      { ...measured, chart: { ...chart, ended_at: chart.started_at } },
    ],
  ] as const) {
    assert.equal(read(usage), undefined, name);
  }
});
