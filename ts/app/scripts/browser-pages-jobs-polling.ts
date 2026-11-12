import assert from "node:assert/strict";

import { parseJobsLive } from "../src/api/jobs-live-parse.ts";
import { jobsHref } from "../src/view/jobs-links.ts";
import { control, facts, visible } from "./browser-fixture.ts";
import type { BrowserResponse } from "./browser-driver.ts";
import { absent, apiAction, capturePage } from "./browser-pages-helpers.ts";
import { workspaceId } from "./fixtures-core.ts";
import { fixtureTemporaryRetryAt } from "./fixtures-jobs-polling.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";

async function readLive(response: BrowserResponse) {
  assert.equal(response.status(), 200);
  assert.equal(response.request().method(), "GET");
  const queue = parseJobsLive(await response.json());
  assert.ok(queue !== undefined, "The browser must read a valid live queue.");
  assert.equal(queue.overflow?.retry_at, fixtureTemporaryRetryAt);
  assert.deepEqual(queue.overflow?.refused_sizes, []);
  const waiting = queue.waiting_labels[0];
  assert.ok(waiting !== undefined && waiting.boat.state === "fitting");
  assert.equal(waiting.boat.refused, false);
  return queue;
}

async function liveReadFacts(origin: string) {
  return (await facts(origin)).filter(
    (fact) => fact.kind === "api" && fact.path.endsWith("/jobs/live"),
  );
}

export function jobsPollingRuns(
  origin: string,
  output: string,
): PagesSmokeRun[] {
  return [
    {
      name: "jobs-temporary-retry-polling",
      cleanup: () => control(origin, { pages: { held: [] } }),
      async check(page, _context, width) {
        await page.clock.pauseAt(new Date("2026-11-06T12:00:05Z"));
        await control(origin, {
          scenario: "m21-jobs-live",
          pages: { jobs: "temporary-retry" },
        });
        const initial = await readLive(
          await apiAction(page, "jobs/live", "GET", async () => {
            await page.goto(origin + jobsHref(workspaceId, "live"), {
              waitUntil: "domcontentloaded",
            });
          }),
        );
        assert.equal(initial.can_manage, true);
        assert.equal(initial.own_slots_total, "6");
        assert.equal(initial.own_slots_used, "6");
        assert.equal(initial.running.length, 6);
        assert.equal(initial.waiting_count, "1");
        assert.equal(initial.waiting.length, 1);
        assert.equal(initial.warm_runners, "0");
        assert.equal(initial.on_boat, "0");
        assert.deepEqual(initial.starting_runners, []);
        assert.deepEqual(
          initial.running.map((job) =>
            job.location?.kind === "machine" ? job.location.first_slot : null,
          ),
          [0, 1, 2, 3, 4, 5],
        );
        await visible(
          page.getByRole("heading", { name: "Running · 6", exact: true }),
        );
        await visible(
          page.getByRole("heading", { name: "Waiting · 1", exact: true }),
        );
        const refused = page.getByText("Boat refused new sandboxes", {
          exact: true,
        });
        const guidance = page.getByText(
          "1 job waits for a free slot. Each one starts on Boat when it has waited 30 seconds.",
          { exact: true },
        );
        await visible(refused);
        await visible(page.getByText(/ciproxy tries Boat again at 12:01/));
        await absent(guidance);
        await capturePage(page, output, "jobs-retry-future", width);

        const readyReads = (await liveReadFacts(origin)).filter(
          (fact) => fact.method === "GET" && fact.status === 200,
        ).length;
        await control(origin, { pages: { held: ["jobs/live"] } });
        const polled = page.waitForRequest(/\/jobs\/live$/);
        await page.clock.runFor(60_000);
        const heldRequest = await polled;
        assert.equal(heldRequest.method(), "GET");
        await visible(refused);
        await absent(guidance);
        const released = page.waitForResponse(
          (response) => response.request() === heldRequest,
        );
        await control(origin, { pages: { held: [] } });
        const refreshed = await released;
        await refreshed.finished();
        const current = await readLive(refreshed);
        assert.equal(current.overflow?.retry_at, initial.overflow?.retry_at);
        await visible(
          page.getByText("Your machines are full", { exact: true }),
        );
        await visible(guidance);
        await absent(refused);
        await capturePage(page, output, "jobs-retry-passed", width);
        const reads = await liveReadFacts(origin);
        assert.ok(
          reads.every((fact) => fact.method === "GET" && fact.status === 200),
        );
        assert.equal(reads.length, readyReads + 1);
      },
    },
  ];
}
