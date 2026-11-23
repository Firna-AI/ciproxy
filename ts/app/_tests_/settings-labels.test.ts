import assert from "node:assert/strict";
import test from "node:test";

import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";

import type {
  LabelWaitReason,
  Problem,
  ProviderHold,
} from "../src/api/schema.ts";
import {
  checkDraft,
  editDraft,
  labelFailure,
  newDraft,
  sizeError,
} from "../src/view/settings-label-form.ts";
import {
  canDelete,
  jobsCaption,
  labelRows,
} from "../src/view/settings-labels.ts";
import { label, labelInput } from "./settings-support.ts";

test("the label list says where jobs run now, as the routing of the API places them, and the jobs of 30 days", () => {
  const waits = (reason: LabelWaitReason) => ({
    routing: { kind: "waits", reason } as const,
  });
  const held = (hold: ProviderHold) =>
    waits({ kind: "provider", provider: "boat", hold });
  const rows = labelRows([
    label("a", { name: "ciproxy" }),
    label(
      "b",
      { name: "own", order: [machinePlace] },
      { fitting_places: [machinePlace] },
    ),
    label("c", { name: "huge" }, { fitting_places: [boatPlace] }),
    label(
      "d",
      { name: "boat-first", order: [boatPlace, machinePlace] },
      { fitting_places: [boatPlace, machinePlace] },
    ),
    label(
      "e",
      { name: "giant", when_nothing_fits: "largest_overflow" },
      { routing: { kind: "largest", provider: "boat" } },
    ),
    label("f", { name: "giant-wait" }, waits({ kind: "nothing_fits" })),
    label("g", { name: "empty", order: [] }, waits({ kind: "empty_order" })),
    label("h", { name: "held-off" }, held("off")),
    label("i", { name: "held-limit" }, held("spend_limit")),
    label("j", { name: "held-new" }, held("not_connected")),
    label("k", { name: "unknown" }, waits({ kind: "unavailable" })),
  ]);
  assert.deepEqual(
    rows.map((row) => [
      row.name,
      row.isDefault,
      row.where.text,
      row.where.warn,
    ]),
    // Every label has the same size, so the list orders them by name.
    [
      ["boat-first", false, "Boat, then Your machines", false],
      ["ciproxy", true, "Your machines, then Boat", false],
      ["empty", false, "Waits: no place", true],
      ["giant", false, "Boat, largest size", false],
      ["giant-wait", false, "Waits: nothing fits", true],
      ["held-limit", false, "Waits: Boat reached its spend limit", true],
      ["held-new", false, "Waits: Boat is not connected", true],
      ["held-off", false, "Waits: overflow providers are off", true],
      ["huge", false, "Boat only", false],
      ["own", false, "Your machines only", false],
      ["unknown", false, "Not known yet", false],
    ],
  );
  const counted = labelRows([
    { ...label("a"), jobs_last_30_days: "14371" },
    { ...label("b"), jobs_last_30_days: "1" },
    { ...label("c"), jobs_last_30_days: "0" },
    { ...label("d"), jobs_last_30_days: null },
    { ...label("e"), jobs_last_30_days: "9007199254740993" },
  ]).map((row) => [row.jobs, jobsCaption(row.jobs)]);
  assert.deepEqual(counted, [
    ["14,371", "14,371 jobs in 30\u00a0days"],
    ["1", "1 job in 30\u00a0days"],
    ["0", "0 jobs in 30\u00a0days"],
    [null, "Job count not known yet"],
    ["9,007,199,254,740,993", "9,007,199,254,740,993 jobs in 30\u00a0days"],
  ]);
});

test("the list shows Linux labels, then macOS labels, from the smallest size to the largest", () => {
  const macos = { platform: "macos_arm64" } as const;
  const medium = { vcpus: 4, memory_gb: 16, disk_gb: 20 };
  const rows = labelRows([
    label("a", { ...macos, ...medium, name: "ciproxy-macos-medium" }),
    label("b", { name: "ciproxy-large" }),
    label("c", { ...macos, name: "ciproxy-macos", vcpus: 2, memory_gb: 8 }),
    label("d", { ...medium, name: "ciproxy-medium" }),
    label("e", { ...medium, name: "more-memory", memory_gb: 32 }),
    label("f", { ...medium, name: "more-disk", disk_gb: 40 }),
    label("g", { ...medium, name: "builds" }),
    label("h", { ...medium, name: "E2E" }),
    label("i", { name: "ciproxy", vcpus: 2, memory_gb: 8, disk_gb: 20 }),
    // Fewer vCPU but more memory than the labels of 4 vCPU, so the vCPU key
    // decides its place.
    label("j", { name: "lean-cpu", vcpus: 2, memory_gb: 64, disk_gb: 20 }),
    // First by name among its memory, but more disk, so the disk key decides.
    label("k", { ...medium, name: "big-disk", disk_gb: 40 }),
  ]);
  assert.deepEqual(
    rows.map((row) => [row.id, row.name]),
    [
      ["i", "ciproxy"],
      ["j", "lean-cpu"],
      // Names of one size compare by character code, so E2E comes first.
      ["h", "E2E"],
      ["g", "builds"],
      ["d", "ciproxy-medium"],
      ["k", "big-disk"],
      ["f", "more-disk"],
      ["e", "more-memory"],
      ["b", "ciproxy-large"],
      ["c", "ciproxy-macos"],
      ["a", "ciproxy-macos-medium"],
    ],
  );
});

test("every label except ciproxy offers Delete label", () => {
  assert.equal(canDelete(labelInput({ name: "ciproxy" })), false);
  assert.equal(canDelete(labelInput({ name: "ciproxy-medium" })), true);
  assert.equal(canDelete(labelInput({ name: "Ciproxy" })), true);
});

test("an edit sends the saved name unchanged with the new size", () => {
  const saved = labelInput({
    name: "ciproxy-medium",
    disk_gb: 20,
    order: [boatPlace, machinePlace],
  });
  const draft = { ...editDraft(saved), disk: "40" };
  assert.deepEqual(checkDraft(draft), {
    ok: true,
    input: { ...saved, platform: "linux_x64", disk_gb: 40 },
  });
});

test("a size needs whole numbers of at least 1", () => {
  const draft = newDraft([], []);
  assert.deepEqual(
    [draft.name, draft.vcpus, draft.memory, draft.disk],
    ["", "", "", ""],
  );
  assert.deepEqual(
    checkDraft({ ...draft, vcpus: "4", memory: "0", disk: "1.5" }),
    { ok: false, errors: { memory: sizeError, disk: sizeError } },
  );
  assert.deepEqual(
    checkDraft({ ...draft, vcpus: "2147483648", memory: "8", disk: "1" }),
    { ok: false, errors: { vcpus: sizeError } },
  );
  const started = newDraft([label("a", { name: "ciproxy", vcpus: 2 })], []);
  assert.deepEqual(
    [started.vcpus, started.memory, started.disk, started.whenNothingFits],
    ["2", "32", "60", "wait"],
  );
});

test("a refusal of GitHub shows under the name, other failures in the form", () => {
  const refusal = {
    code: "github_label_refused",
    message: "Label names cannot contain spaces.",
  } satisfies Problem;
  assert.deepEqual(
    labelFailure({ kind: "problem", status: 422, problem: refusal }, "save"),
    { kind: "name", message: "Label names cannot contain spaces." },
  );
  assert.deepEqual(labelFailure({ kind: "network" }, "delete"), {
    kind: "form",
    copy: {
      title: "Unable to delete the label",
      body: "Check the connection, then try again.",
    },
  });
  const busy = { code: "unavailable", message: "Try again." } satisfies Problem;
  assert.deepEqual(
    labelFailure({ kind: "problem", status: 503, problem: busy }, "save"),
    {
      kind: "form",
      copy: {
        title: "Unable to save the label",
        body: "ciproxy could not reach GitHub or your machines. Try again.",
      },
    },
  );
});
