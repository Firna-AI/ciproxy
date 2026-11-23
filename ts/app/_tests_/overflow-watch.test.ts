import assert from "node:assert/strict";
import test from "node:test";

import type { OverflowProvider } from "../src/api/schema.ts";
import type { OverflowStore } from "../src/state/overflow.ts";
import { offeredProviders } from "../src/view/settings-label-places.ts";
import { workspaceId } from "../scripts/fixtures-core.ts";
import { notConnected, overflowSettings } from "../scripts/fixtures-boat.ts";
import { appWith, response, settled } from "./support.ts";

/** The providers that Add a place offers, or null while none are known. */
function offered(store: OverflowStore): OverflowProvider[] | null {
  const state = store.state.get();
  return state.status === "ready"
    ? offeredProviders(state.settings.providers)
    : null;
}

test("a watch reads the overflow settings at once and each time the tab shows again, so Add a place offers a provider connected in another tab", async () => {
  const { app, calls, schedule } = appWith(
    response(notConnected()),
    response(overflowSettings()),
  );
  const store = app.overflow(workspaceId);
  const stop = store.watch();
  await settled();
  assert.deepEqual(offered(store), [], "the dialog opens before Boat connects");
  schedule.hide(true);
  await settled();
  assert.equal(calls.length, 1, "a hidden tab reads nothing");
  schedule.hide(false);
  await settled();
  assert.deepEqual(
    offered(store),
    ["boat"],
    "the tab shows again after Boat connected in another tab",
  );
  stop();
  schedule.hide(true);
  schedule.hide(false);
  await settled();
  assert.equal(calls.length, 2, "a closed dialog reads nothing");
});
