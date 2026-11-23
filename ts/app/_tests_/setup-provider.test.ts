import assert from "node:assert/strict";
import test from "node:test";

import { createProviderSkipStore } from "../src/state/provider-skip.ts";
import { memoryStorage } from "../src/state/storage.ts";
import {
  setupComplete,
  setupModel,
  type ProviderProgress,
} from "../src/view/setup-steps.ts";
import { otherWorkspaceId, workspaceId } from "../scripts/fixtures-core.ts";
import { job, machine, setup } from "../scripts/fixtures-setup.ts";

const none: ProviderProgress = { connected: null, skipped: false };
const skipped: ProviderProgress = { connected: null, skipped: true };
const connected: ProviderProgress = {
  connected:
    "Connected to the Boat account northstar · spend limit $200 a month",
  skipped: false,
};
const joined = setup({ machines: 1, waiting_for_machine: false });

function statuses(provider: ProviderProgress, view = joined, hold = false) {
  const model = setupModel(view, [machine()], hold, provider);
  return {
    steps: model.steps.map((step) => `${step.key}:${step.status}`),
    done: model.done,
    complete: model.complete,
  };
}

test("the provider step comes after the machine, and the first job opens after a connection or a skip", () => {
  assert.deepEqual(statuses(none), {
    steps: ["github:done", "machine:done", "provider:active", "job:todo"],
    done: 2,
    complete: false,
  });
  assert.deepEqual(statuses(connected), {
    steps: ["github:done", "machine:done", "provider:done", "job:active"],
    done: 3,
    complete: false,
  });
  assert.deepEqual(statuses(skipped), {
    steps: ["github:done", "machine:done", "provider:skipped", "job:active"],
    done: 2,
    complete: false,
  });
  assert.equal(
    setupModel(joined, [machine()], false, connected).steps[2]?.summary,
    connected.connected,
  );
  assert.deepEqual(statuses(none, joined, true).steps, [
    "github:done",
    "machine:active",
    "provider:todo",
    "job:todo",
  ]);
});

test("a running first job keeps its step open, and Setup completes with the first job alone", () => {
  const running = setup({ ...joined, first_job: job() });
  assert.deepEqual(statuses(none, running).steps, [
    "github:done",
    "machine:done",
    "provider:active",
    "job:active",
  ]);
  const finished = setup({
    ...joined,
    first_job: job({ phase: "completed", result: "success" }),
  });
  assert.deepEqual(statuses(none, finished), {
    steps: ["github:done", "machine:done", "provider:active", "job:done"],
    done: 3,
    complete: true,
  });
  assert.equal(setupComplete(finished, [machine()]), true);
  assert.equal(setupComplete(running, [machine()]), false);
});

test("a skip lasts for the tab, for one workspace and the person who skipped", () => {
  const tab = memoryStorage();
  const first = createProviderSkipStore(tab, workspaceId);
  assert.equal(first.state.get(), null);
  first.skip("mira");
  assert.equal(first.state.get(), "mira");
  assert.equal(createProviderSkipStore(tab, workspaceId).state.get(), "mira");
  assert.equal(
    createProviderSkipStore(tab, otherWorkspaceId).state.get(),
    null,
  );
  assert.equal(
    createProviderSkipStore(memoryStorage(), workspaceId).state.get(),
    null,
  );
});
