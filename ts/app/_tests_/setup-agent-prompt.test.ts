import assert from "node:assert/strict";
import test from "node:test";

import type { LabelInput, LabelView } from "../src/api/schema.ts";
import { agentPrompt } from "../src/view/setup-agent-prompt.ts";
import { machine, setup } from "../scripts/fixtures-setup.ts";

const [base] = setup().labels;

function label(fields: Partial<LabelInput>): LabelView {
  assert.ok(base);
  return { ...base, label: { ...base.label, ...fields } };
}

const defaults: readonly LabelView[] = [
  label({ name: "ciproxy-large", vcpus: 8, memory_gb: 32, disk_gb: 60 }),
  label({
    name: "ciproxy-macos",
    platform: "macos_arm64",
    allow_boat: false,
    vcpus: 2,
    memory_gb: 8,
  }),
  label({ name: "ciproxy", vcpus: 2, memory_gb: 8 }),
  label({ name: "ciproxy-medium", vcpus: 4, memory_gb: 16 }),
];

const mac = machine({ hostname: "studio-01", platform: "macos_arm64" });

/** The label lines of a prompt, in their order. */
function labelLines(prompt: string | null): string[] {
  assert.ok(prompt !== null);
  return prompt.split("\n").filter((line) => line.startsWith("- "));
}

test("The agent prompt lists the Linux labels by size without a Mac", () => {
  const prompt = agentPrompt(defaults, [machine()]);
  assert.deepEqual(labelLines(prompt), [
    "- ciproxy: Linux, 2 vCPU, 8 GB memory, 20 GB free disk",
    "- ciproxy-medium: Linux, 4 vCPU, 16 GB memory, 20 GB free disk",
    "- ciproxy-large: Linux, 8 vCPU, 32 GB memory, 60 GB free disk",
  ]);
  assert.match(prompt ?? "", /runs on Windows, on macOS, or on Linux on Arm\./);
});

test("The agent prompt lists macOS labels while a Mac can start VMs", () => {
  const prompt = agentPrompt(defaults, [machine(), mac]);
  assert.deepEqual(labelLines(prompt).slice(3), [
    "- ciproxy-macos: macOS, 2 vCPU, 8 GB memory, 20 GB free disk",
  ]);
  assert.match(prompt ?? "", /runs on Windows, or on Linux on Arm\./);
  const blocked = machine({
    hostname: "studio-01",
    platform: "macos_arm64",
    virtualization: "entitlement_missing",
  });
  assert.equal(labelLines(agentPrompt(defaults, [blocked])).length, 3);
});

test("The agent prompt leaves out a label that can run nowhere", () => {
  const nowhere = label({
    name: "ciproxy-off",
    allow_boat: false,
    allow_machines: false,
  });
  const boatOnly = label({ name: "ciproxy-boat", allow_machines: false });
  assert.deepEqual(labelLines(agentPrompt([nowhere, boatOnly], [])), [
    "- ciproxy-boat: Linux, 4 vCPU, 8 GB memory, 20 GB free disk",
  ]);
  assert.equal(agentPrompt([nowhere], []), null);
});
