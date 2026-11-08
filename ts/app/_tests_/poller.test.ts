import assert from "node:assert/strict";
import test from "node:test";

import { createPoller } from "../src/state/poller.ts";
import { deferred, manualSchedule, settled } from "./support.ts";

test("polling pauses while hidden, resumes once, and never overlaps reads", async () => {
  const schedule = manualSchedule();
  const read = deferred<boolean>();
  let calls = 0;
  const poller = createPoller(
    () => {
      calls += 1;
      return read.promise;
    },
    5000,
    schedule.timers,
    schedule.visibility,
  );
  poller.start();
  poller.start();
  assert.deepEqual(schedule.delays, [5000]);
  schedule.hide(true);
  assert.equal(schedule.pending(), 0);
  schedule.hide(false);
  assert.equal(calls, 1);
  schedule.hide(false);
  assert.equal(calls, 1);
  assert.equal(schedule.pending(), 0);
  read.resolve(true);
  await read.promise;
  await settled();
  assert.equal(schedule.pending(), 1);
  poller.stop();
  assert.equal(schedule.pending(), 0);
  schedule.hide(false);
  assert.equal(calls, 1);
});

test("polling stops at a terminal result and restarts only when requested", async () => {
  const schedule = manualSchedule();
  let calls = 0;
  const poller = createPoller(
    async () => {
      calls += 1;
      return false;
    },
    3000,
    schedule.timers,
    schedule.visibility,
  );
  poller.start();
  schedule.fire();
  await settled();
  assert.equal(calls, 1);
  assert.equal(schedule.pending(), 0);
  schedule.hide(false);
  assert.equal(calls, 1);
  poller.start();
  assert.equal(schedule.pending(), 1);
  poller.stop();
});
