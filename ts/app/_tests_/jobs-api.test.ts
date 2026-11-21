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
  const detail = parseJobDetail({ job, banner });
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
    parseJobDetail({ job: { ...job, runtime_seconds: "1e3" }, banner: null }),
    undefined,
  );
  assert.equal(parseSavings({ ...report, from: "2026-02-30" }), undefined);
});
