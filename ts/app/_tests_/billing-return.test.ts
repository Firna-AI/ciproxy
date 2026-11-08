import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type {
  CheckoutReturnPage,
  PaymentOutcome,
} from "../src/api/billing-types.ts";
import {
  confirmingAnswers,
  createReturnStore,
  returnPollInterval,
} from "../src/state/billing-return.ts";
import { conflict, unavailable } from "./billing-support.ts";
import { deferred, manualSchedule, settled } from "./support.ts";

type Reply =
  ApiResult<CheckoutReturnPage> | Promise<ApiResult<CheckoutReturnPage>>;

function answer(outcome: PaymentOutcome): ApiResult<CheckoutReturnPage> {
  return { ok: true, value: { outcome, banner: null } };
}

function refused(
  status: number,
  problem: typeof conflict | typeof unavailable,
) {
  return {
    ok: false,
    failure: { kind: "problem", status, problem },
  } satisfies ApiResult<CheckoutReturnPage>;
}

function returnWith(...replies: Reply[]) {
  const schedule = manualSchedule();
  let checks = 0;
  let reads = 0;
  const store = createReturnStore({
    reconcile: async () => {
      checks += 1;
      const reply = replies.shift();
      assert.notEqual(reply, undefined, "Unexpected return check");
      if (reply === undefined) throw new Error("Missing reply");
      return reply;
    },
    mutate: (run) => run(),
    timers: schedule.timers,
    visibility: schedule.visibility,
    settled: async () => {
      reads += 1;
    },
  });
  return { store, schedule, checks: () => checks, reads: () => reads };
}

test("a return page that closes stops checking, also with a check on its way", async () => {
  const held = deferred<ApiResult<CheckoutReturnPage>>();
  const { store, schedule, checks } = returnWith(held.promise);
  store.start();
  store.stop();
  held.resolve(answer({ state: "confirming" }));
  await settled();
  assert.equal(schedule.pending(), 0);
  schedule.hide(false);
  await settled();
  assert.equal(checks(), 1);
});

test("the return waits for the server, says that Stripe still confirms, then settles once", async () => {
  const { store, schedule, checks, reads } = returnWith(
    answer({ state: "confirming" }),
    refused(409, conflict),
    answer({ state: "confirming" }),
    answer({ state: "confirmed" }),
  );
  store.start();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "confirming",
    attempts: 1,
    still: false,
  });
  assert.deepEqual(schedule.delays, [returnPollInterval]);
  schedule.fire();
  await settled();
  schedule.fire();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "confirming",
    attempts: confirmingAnswers,
    still: true,
  });
  assert.equal(reads(), 0);
  schedule.fire();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "settled",
    outcome: { state: "confirmed" },
  });
  assert.equal(reads(), 1);
  assert.equal(schedule.pending(), 0);
  assert.equal(checks(), 4);
});

test("a failed check stops, shows the failure, and Try again checks the same session", async () => {
  const { store, schedule, checks } = returnWith(
    refused(503, unavailable),
    answer({ state: "unfinished" }),
  );
  store.start();
  await settled();
  const failed = store.state.get();
  assert.equal(failed.status, "failed");
  assert.equal(schedule.pending(), 0);
  store.retry();
  await settled();
  assert.deepEqual(store.state.get(), {
    status: "settled",
    outcome: { state: "unfinished" },
  });
  assert.equal(checks(), 2);
});
