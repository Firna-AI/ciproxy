import assert from "node:assert/strict";

import { createBannerEndpoint } from "../src/api/banner.ts";
import { createBillingEndpoints } from "../src/api/billing.ts";
import type { ApiClient } from "../src/api/client.ts";
import { createJobsEndpoints } from "../src/api/jobs.ts";
import { createSavingsEndpoints } from "../src/api/savings.ts";
import { apiValue, type LiveChecks } from "./live-pages-checks.ts";

export async function readLivePages(
  api: ApiClient,
  workspaceId: string,
  checks: LiveChecks,
): Promise<void> {
  const jobs = createJobsEndpoints(api);
  const billing = createBillingEndpoints(api);
  const live = await checks.check("Jobs live response", async () =>
    apiValue(await jobs.live(workspaceId)),
  );
  const history = await checks.check("Jobs history response", async () =>
    apiValue(await jobs.history(workspaceId, {})),
  );
  const job = history.jobs[0] ?? live.running[0] ?? live.waiting[0];
  if (job === undefined) checks.skip("job detail without a visible job");
  else
    await checks.check("exact visible job detail", async () => {
      const detail = apiValue(await jobs.detail(workspaceId, job.id));
      assert.equal(detail.job.id, job.id);
    });
  for (const range of ["last30_days", "this_month", "last_month"] as const) {
    await checks.check(`Savings ${range} response`, async () => {
      const report = apiValue(
        await createSavingsEndpoints(api).report(workspaceId, range),
      );
      assert.equal(report.range, range);
    });
  }
  await checks.check("Plan and usage response", async () =>
    apiValue(await billing.usage(workspaceId)),
  );
  await checks.check("Plans response", async () =>
    apiValue(await billing.plans(workspaceId)),
  );
  await checks.check("page banner response", async () =>
    apiValue(await createBannerEndpoint(api).banner(workspaceId)),
  );
}
