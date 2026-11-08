import assert from "node:assert/strict";
import test from "node:test";

import { draftText, settleDraft } from "../src/state/saved-draft.ts";

test("a typed limit stays when the saved limit changes, also at the old saved limit or empty, while an untouched field follows it", () => {
  assert.equal(draftText(null, "200"), "200");
  assert.equal(draftText(null, "100"), "100");
  assert.equal(draftText("200", "100"), "200");
  assert.equal(draftText("", "100"), "");
});

test("a failed save keeps the typed limit, and an accepted save of its form shows the saved limit again", () => {
  assert.equal(draftText(settleDraft("1,000", false), "200"), "1,000");
  const accepted = settleDraft("1,000", true);
  assert.equal(draftText(accepted, "1000"), "1000");
  assert.equal(draftText(accepted, "150"), "150");
});
