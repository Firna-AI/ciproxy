import assert from "node:assert/strict";
import test from "node:test";

import { afterText, copyState } from "../src/components/copy-state.ts";

test("copy feedback belongs only to the current text", () => {
  const oldText = "revoked install command";
  const replacement = "replacement install command";
  assert.equal(copyState(null, replacement), "idle");
  assert.equal(copyState({ text: oldText, copied: true }, oldText), "copied");
  assert.equal(copyState({ text: oldText, copied: true }, replacement), "idle");
  assert.equal(
    copyState({ text: oldText, copied: false }, replacement),
    "idle",
  );
  assert.equal(
    copyState({ text: replacement, copied: true }, replacement),
    "copied",
  );
  assert.equal(
    copyState({ text: replacement, copied: false }, replacement),
    "failed",
  );
});

test("workflow Copy produces usable YAML without deleted lines or diff markers", () => {
  assert.equal(
    afterText([
      "  jobs:",
      "    test:",
      "-     runs-on: ubuntu-latest",
      "+     runs-on: ciproxy",
    ]),
    "jobs:\n  test:\n    runs-on: ciproxy",
  );
});
