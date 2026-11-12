import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import { createJobsEndpoints } from "../src/api/jobs.ts";
import type { HistoryQuery, JobsHistoryPage } from "../src/api/jobs-types.ts";
import type { Problem } from "../src/api/schema.ts";
import { createHistoryStore } from "../src/state/jobs-history.ts";
import { createLiveStore } from "../src/state/jobs-live.ts";
import { session, workspaceId } from "../scripts/fixtures-core.ts";
import { fixtureJob } from "../scripts/fixtures-jobs.ts";
import { liveJobs } from "../scripts/fixtures-jobs-live.ts";
import { appWith, deferred, response, settled } from "./support.ts";

const unavailable = {
  code: "unavailable",
  message: "Try again.",
} satisfies Problem;

test("a failed live refresh keeps the last queue with its failure, and a 401 ends the sign-in", async () => {
  const first = liveJobs("live", "admin", null);
  const { app } = appWith(
    response(session()),
    response(first),
    response(unavailable, 503),
    response({ code: "sign_in_required", message: "Sign in." }, 401),
  );
  await app.session.ensure();
  const endpoints = createJobsEndpoints(app.api);
  const store = createLiveStore({
    read: () => endpoints.live(workspaceId),
    report: app.session.report,
    now: app.env.now,
    timers: app.env.timers,
    visibility: app.env.visibility,
  });
  await store.load();
  await store.load();
  const kept = store.state.get();
  assert.deepEqual(
    kept.status === "ready"
      ? [kept.page.running.length, kept.refreshFailure]
      : kept.status,
    [
      first.running.length,
      { kind: "problem", status: 503, problem: unavailable },
    ],
  );
  await store.load();
  await settled();
  assert.equal(app.session.state.get().status, "signed-out");
});

test("a late answer of an older History filter does not replace the newer list", async () => {
  const answers: {
    query: HistoryQuery;
    reply: (page: JobsHistoryPage) => void;
  }[] = [];
  const store = createHistoryStore({
    read: (query) => {
      const answer = deferred<ApiResult<JobsHistoryPage>>();
      answers.push({
        query,
        reply: (page) => answer.resolve({ ok: true, value: page }),
      });
      return answer.promise;
    },
    report: (result) => result,
  });
  const old = store.load({ from: "2026-11-06" });
  const current = store.load({ from: "2026-11-06", search: "deploy" });
  const page = (name: string): JobsHistoryPage => ({
    jobs: [fixtureJob({ name })],
    history_days: 30,
    banner: null,
  });
  answers[1]?.reply(page("deploy preview"));
  await current;
  answers[0]?.reply(page("test (rust)"));
  await old;
  const state = store.state.get();
  assert.deepEqual(
    state.status === "ready"
      ? [state.query, state.page.jobs.map((job) => job.name)]
      : state.status,
    [{ from: "2026-11-06", search: "deploy" }, ["deploy preview"]],
  );
});
