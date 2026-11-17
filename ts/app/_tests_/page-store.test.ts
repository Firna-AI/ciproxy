import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import { createPageStore } from "../src/state/page-store.ts";
import { manualSchedule, settled } from "./support.ts";

type Page = { waits: boolean };

const failed: ApiResult<Page> = { ok: false, failure: { kind: "network" } };

function storeWith(...replies: ApiResult<Page>[]) {
  const schedule = manualSchedule();
  const store = createPageStore<Page>({
    read: async () => {
      const reply = replies.shift();
      assert.notEqual(reply, undefined, "Unexpected read");
      return reply ?? failed;
    },
    report: (result) => result,
    waits: (page) => page.waits,
    interval: 7_000,
    timers: schedule.timers,
    visibility: schedule.visibility,
  });
  return { store, schedule, replies };
}

test("a watched page reads again after a failed read until a good page that does not wait", async () => {
  const { store, schedule, replies } = storeWith(failed, failed, {
    ok: true,
    value: { waits: false },
  });
  await store.load();
  const stop = store.watch();
  assert.deepEqual(schedule.delays, [7_000]);
  schedule.fire();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "failed",
    failure: { kind: "network" },
  });
  assert.equal(schedule.pending(), 1);
  schedule.fire();
  await settled();
  stop();
  assert.deepEqual(store.state.get(), {
    status: "ready",
    page: { waits: false },
    refreshFailure: null,
  });
  assert.equal(schedule.pending(), 0);
  assert.deepEqual(schedule.delays, [7_000, 7_000]);
  assert.equal(replies.length, 0);
});
