import assert from "node:assert/strict";
import test from "node:test";

import type { ApiResult } from "../src/api/client.ts";
import type { LabelFit, LabelInput } from "../src/api/schema.ts";
import {
  createLabelPreview,
  previewDelay,
} from "../src/state/settings-preview.ts";
import { deferred, manualSchedule, settled } from "./support.ts";
import { fit, labelInput } from "./settings-support.ts";

function previewWith() {
  const schedule = manualSchedule();
  const asked: LabelInput[] = [];
  const answers: ReturnType<typeof deferred<ApiResult<LabelFit>>>[] = [];
  const preview = createLabelPreview({
    preview: (input) => {
      asked.push(input);
      const answer = deferred<ApiResult<LabelFit>>();
      answers.push(answer);
      return answer.promise;
    },
    timers: schedule.timers,
    initial: null,
  });
  return { preview, schedule, asked, answers };
}

test("the preview asks once after the pause, for the latest draft", () => {
  const { preview, schedule, asked } = previewWith();
  preview.request(labelInput({ vcpus: 4 }));
  preview.request(labelInput({ vcpus: 8 }));
  assert.equal(asked.length, 0);
  assert.deepEqual(schedule.delays, [previewDelay, previewDelay]);
  assert.equal(schedule.pending(), 1);
  schedule.fire();
  assert.deepEqual(
    asked.map((input) => input.vcpus),
    [8],
  );
  assert.equal(preview.state.get().pending, true);
});

test("a late answer for an older draft never replaces a newer one", async () => {
  const { preview, schedule, asked, answers } = previewWith();
  const older = labelInput({ vcpus: 4 });
  const newer = labelInput({ vcpus: 8 });
  preview.request(older);
  schedule.fire();
  preview.request(newer);
  schedule.fire();
  answers[1]?.resolve({ ok: true, value: fit({ maximum_jobs: "1" }) });
  await settled();
  answers[0]?.resolve({ ok: true, value: fit({ maximum_jobs: "3" }) });
  await settled();
  const newerShown = {
    shown: { input: newer, fit: fit({ maximum_jobs: "1" }) },
    pending: false,
    failure: null,
  };
  assert.deepEqual(preview.state.get(), newerShown);
  // The shown size comes back while the ask for another size is in flight.
  preview.request(labelInput({ vcpus: 80 }));
  schedule.fire();
  preview.request(newer);
  answers[2]?.resolve({ ok: true, value: fit({ maximum_jobs: "0" }) });
  await settled();
  assert.deepEqual(preview.state.get(), newerShown);
  // The shown size comes back while another size waits for its pause.
  preview.request(labelInput({ vcpus: 80 }));
  preview.request(newer);
  assert.equal(schedule.pending(), 0);
  assert.deepEqual(preview.state.get(), newerShown);
  assert.deepEqual(
    asked.map((input) => input.vcpus),
    [4, 8, 80],
  );
});

test("a draft with no valid size and a closed dialog ask for nothing", async () => {
  const { preview, schedule, asked, answers } = previewWith();
  preview.request(labelInput());
  preview.request(null);
  assert.equal(schedule.pending(), 0);
  assert.equal(preview.state.get().pending, false);
  preview.request(labelInput());
  schedule.fire();
  preview.stop();
  answers[0]?.resolve({ ok: true, value: fit() });
  await settled();
  assert.equal(asked.length, 1);
  assert.equal(preview.state.get().shown, null);
});

test("a change of the Boat switch or of When nothing fits asks again, and a new name does not", async () => {
  const { preview, schedule, asked, answers } = previewWith();
  const saved = labelInput();
  preview.request(saved);
  schedule.fire();
  answers[0]?.resolve({ ok: true, value: fit() });
  await settled();
  preview.request({ ...saved, name: "renamed-draft" });
  assert.equal(schedule.pending(), 0, "a new name reuses the shown fit");
  preview.request({ ...saved, allow_boat: false });
  schedule.fire();
  preview.request({ ...saved, when_nothing_fits: "largest_boat" });
  schedule.fire();
  assert.deepEqual(
    asked.map((input) => [input.allow_boat, input.when_nothing_fits]),
    [
      [true, "wait"],
      [false, "wait"],
      [true, "largest_boat"],
    ],
  );
  const fallback = fit({ placement: "boat_only" });
  answers[2]?.resolve({ ok: true, value: fallback });
  await settled();
  assert.deepEqual(preview.state.get(), {
    shown: {
      input: { ...saved, when_nothing_fits: "largest_boat" },
      fit: fallback,
    },
    pending: false,
    failure: null,
  });
});
