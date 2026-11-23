import assert from "node:assert/strict";
import test from "node:test";

import type {
  LabelInput,
  LabelPlacement,
  LabelView,
} from "../src/api/schema.ts";
import { agentPrompt } from "../src/view/setup-agent-prompt.ts";
import { setup } from "../scripts/fixtures-setup.ts";

const [base] = setup().labels;

function label(
  fields: Partial<LabelInput>,
  placement: LabelPlacement = "machines_only",
): LabelView {
  assert.ok(base);
  return {
    ...base,
    fit: { ...base.fit, placement },
    label: { ...base.label, ...fields },
  };
}

/** The label lines of a prompt, in their order. */
function labelLines(prompt: string | null): string[] {
  assert.ok(prompt !== null);
  return prompt.split("\n").filter((line) => line.startsWith("- "));
}

test("The agent prompt lists the labels by platform and size", () => {
  const prompt = agentPrompt([
    label({ name: "ciproxy-large", vcpus: 8, memory_gb: 32, disk_gb: 60 }),
    label({ name: "ciproxy-macos", platform: "macos_arm64", vcpus: 2 }),
    label({ name: "ciproxy", vcpus: 2, memory_gb: 8 }),
    label({ name: "ciproxy-medium", vcpus: 4, memory_gb: 16 }, "boat_only"),
  ]);
  assert.deepEqual(labelLines(prompt), [
    "- ciproxy: Linux on x64, 2 vCPU, 8 GB memory, 20 GB free disk",
    "- ciproxy-medium: Linux on x64, 4 vCPU, 16 GB memory, 20 GB free disk",
    "- ciproxy-large: Linux on x64, 8 vCPU, 32 GB memory, 60 GB free disk",
    "- ciproxy-macos: macOS on Apple silicon (arm64), 2 vCPU, 8 GB memory, 20 GB free disk",
  ]);
});

test("The agent prompt leaves out a label that no destination fits", () => {
  const prompt = agentPrompt([
    label({ name: "ciproxy", vcpus: 2 }, "unavailable"),
    label(
      { name: "ciproxy-macos-medium", platform: "macos_arm64", vcpus: 4 },
      "waits",
    ),
  ]);
  assert.deepEqual(labelLines(prompt), [
    "- ciproxy: Linux on x64, 2 vCPU, 8 GB memory, 20 GB free disk",
  ]);
  assert.equal(agentPrompt([label({ name: "ciproxy" }, "waits")]), null);
});

test("The agent prompt leaves out a label that only a smaller Boat size runs", () => {
  const large = label(
    { name: "ciproxy-large", vcpus: 8, memory_gb: 32 },
    "boat_only",
  );
  const size = {
    vcpus: 2,
    memory_bytes: "8000000000",
    disk_bytes: "40000000000",
  };
  const fallback: LabelView = {
    ...large,
    fit: {
      ...large.fit,
      boat: { state: "largest_fallback", refused: false, size, size_name: "s" },
    },
  };
  const fitting: LabelView = {
    ...large,
    fit: {
      ...large.fit,
      boat: { state: "fitting", refused: false, size, size_name: "s" },
    },
  };
  assert.equal(agentPrompt([fallback]), null);
  assert.deepEqual(labelLines(agentPrompt([fitting])), [
    "- ciproxy-large: Linux on x64, 8 vCPU, 32 GB memory, 20 GB free disk",
  ]);
});

test("The matrix example of the agent prompt names no runner or label", () => {
  const prompt = agentPrompt([
    label({ name: "ciproxy-macos", platform: "macos_arm64" }),
  ]);
  assert.match(
    prompt ?? "",
    /runs-on: \$\{\{ matrix\.os == '<runner>' && '<label>' \|\| matrix\.os \}\}/,
  );
  assert.doesNotMatch(prompt ?? "", /ubuntu-latest/);
});
