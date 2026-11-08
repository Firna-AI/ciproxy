import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type { SavingsPage, SavingsRange } from "../src/api/savings-types.ts";
import type { Visibility } from "../src/state/poller.ts";
import { createSavingsStore } from "../src/state/savings.ts";
import { savingsReport } from "../scripts/fixtures-savings.ts";
import { deferred, manualSchedule, settled } from "./support.ts";

/** Reads that wait until the test answers them, in any order. */
function heldReads() {
  const calls: {
    range: SavingsRange;
    answer: (page: SavingsPage) => void;
  }[] = [];
  const read = (range: SavingsRange) => {
    const reply = deferred<ApiResult<SavingsPage>>();
    calls.push({
      range,
      answer: (page) => reply.resolve({ ok: true, value: page }),
    });
    return reply.promise;
  };
  const answer = (index: number, page: SavingsPage) => {
    const call = calls[index];
    assert.notEqual(call, undefined, `Expected read ${index}`);
    call?.answer(page);
  };
  return { calls, read, answer };
}

/** The visibility of the manual schedule, with a count of its tab watchers. */
function countedWatchers(visibility: Visibility) {
  let watchers = 0;
  const counted: Visibility = {
    hidden: visibility.hidden,
    watch(listener) {
      watchers += 1;
      const remove = visibility.watch(listener);
      return () => {
        watchers -= 1;
        remove();
      };
    },
  };
  return { visibility: counted, count: () => watchers };
}

const incomplete = (range: SavingsRange) =>
  savingsReport("refresh-incomplete", range, true, null);
const complete = (range: SavingsRange) =>
  savingsReport("report", range, true, null);

for (const order of ["after", "before"] as const) {
  test(`a stale poll of the old range that answers ${order} the new range keeps the new range refreshing`, async () => {
    const schedule = manualSchedule();
    const reads = heldReads();
    const store = createSavingsStore({
      read: reads.read,
      report: (result) => result,
      timers: schedule.timers,
      visibility: schedule.visibility,
    });
    store.open();
    const first = store.select("last30_days");
    reads.answer(0, incomplete("last30_days"));
    await first;
    schedule.fire();
    await settled();
    const second = store.select("this_month");
    await settled();
    if (order === "before") reads.answer(1, complete("last30_days"));
    reads.answer(2, incomplete("this_month"));
    await second;
    if (order === "after") reads.answer(1, complete("last30_days"));
    await settled();
    assert.equal(schedule.pending(), 1, "the new range has a refresh");
    schedule.fire();
    await settled();
    reads.answer(3, complete("this_month"));
    await settled();
    const state = store.state.get();
    assert.deepEqual(
      [
        reads.calls.map((call) => call.range),
        state.status === "ready" ? state.range : state.status,
        state.status === "ready" ? state.page.refresh_complete : null,
        schedule.pending(),
      ],
      [
        ["last30_days", "last30_days", "this_month", "this_month"],
        "this_month",
        true,
        0,
      ],
    );
  });
}

test("a new range after a finished refresh keeps one tab watcher", async () => {
  const schedule = manualSchedule();
  const reads = heldReads();
  const watched = countedWatchers(schedule.visibility);
  const store = createSavingsStore({
    read: reads.read,
    report: (result) => result,
    timers: schedule.timers,
    visibility: watched.visibility,
  });
  store.open();
  const first = store.select("last30_days");
  reads.answer(0, incomplete("last30_days"));
  await first;
  schedule.fire();
  await settled();
  reads.answer(1, complete("last30_days"));
  await settled();
  const second = store.select("last_month");
  reads.answer(2, incomplete("last_month"));
  await second;
  assert.deepEqual([watched.count(), schedule.pending()], [1, 1]);
});

test("a page that closes before an incomplete report answers starts no refresh and keeps no tab watcher", async () => {
  const schedule = manualSchedule();
  const reads = heldReads();
  const watched = countedWatchers(schedule.visibility);
  const store = createSavingsStore({
    read: reads.read,
    report: (result) => result,
    timers: schedule.timers,
    visibility: watched.visibility,
  });
  const close = store.open();
  const pending = store.select("last30_days");
  close();
  reads.answer(0, incomplete("last30_days"));
  await pending;
  await settled();
  assert.deepEqual(
    [reads.calls.length, schedule.pending(), watched.count()],
    [1, 0, 0],
  );
});

test("a page that closes during a refresh ends the refresh and its tab watcher", async () => {
  const schedule = manualSchedule();
  const reads = heldReads();
  const watched = countedWatchers(schedule.visibility);
  const store = createSavingsStore({
    read: reads.read,
    report: (result) => result,
    timers: schedule.timers,
    visibility: watched.visibility,
  });
  const close = store.open();
  const first = store.select("last30_days");
  reads.answer(0, incomplete("last30_days"));
  await first;
  const refreshing = [schedule.pending(), watched.count()];
  close();
  assert.deepEqual(
    [refreshing, [schedule.pending(), watched.count()]],
    [
      [1, 1],
      [0, 0],
    ],
  );
});
