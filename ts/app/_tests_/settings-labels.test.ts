import assert from "node:assert/strict";
import test from "node:test";

import type { Problem } from "../src/api/schema.ts";
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

test("the label list says where jobs run now, as the API places them, and the jobs of 30 days", () => {
  const noBoat = {
    boat: { state: "unavailable" },
    boat_connected: false,
  } as const;
  const rows = labelRows([
    label("a", { name: "ciproxy" }),
    label(
      "b",
      { name: "own", allow_boat: false },
      { placement: "machines_only" },
    ),
    label("c", { name: "huge" }, { placement: "boat_only" }),
    label("d", { name: "giant" }, { placement: "waits" }),
    label(
      "e",
      { name: "giant-own", allow_boat: false },
      { placement: "waits" },
    ),
    label(
      "f",
      { name: "giant-boat", allow_machines: false },
      { placement: "waits" },
    ),
    label(
      "g",
      { name: "off", allow_machines: false, allow_boat: false },
      { placement: "waits" },
    ),
    label("h", { name: "new-box" }, { placement: "unavailable" }),
    label(
      "i",
      { name: "unknown" },
      { placement: "unavailable", fitting_machines: 0 },
    ),
    label("j", { name: "no-boat" }, { ...noBoat, placement: "machines_only" }),
    label("k", { name: "no-boat-big" }, { ...noBoat, placement: "waits" }),
    label(
      "l",
      { name: "no-boat-only", allow_machines: false },
      { ...noBoat, placement: "waits" },
    ),
  ]);
  assert.deepEqual(
    rows.map((row) => [
      row.name,
      row.isDefault,
      row.where.text,
      row.where.warn,
      row.where.note,
    ]),
    [
      ["ciproxy", true, "Your machines, then Boat", false, null],
      ["own", false, "Your machines only", false, null],
      ["huge", false, "Boat only", false, null],
      ["giant", false, "Waits: nothing fits", true, null],
      ["giant-own", false, "Waits: no machine fits", true, null],
      ["giant-boat", false, "Waits: no Boat size fits", true, null],
      ["off", false, "Waits: Your machines and Boat are off", true, null],
      ["new-box", false, "Your machines", false, "Boat fit not known yet"],
      ["unknown", false, "Not known yet", false, null],
      ["no-boat", false, "Your machines only", false, null],
      ["no-boat-big", false, "Waits: no machine fits", true, null],
      ["no-boat-only", false, "Waits: Boat is not connected", true, null],
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

test("every label except ciproxy offers Delete label", () => {
  assert.equal(canDelete(labelInput({ name: "ciproxy" })), false);
  assert.equal(canDelete(labelInput({ name: "ciproxy-4vcpu" })), true);
  assert.equal(canDelete(labelInput({ name: "Ciproxy" })), true);
});

test("an edit sends the saved name unchanged with the new size", () => {
  const saved = labelInput({ name: "ciproxy-4vcpu", disk_gb: 20 });
  const draft = { ...editDraft(saved), disk: "40" };
  assert.deepEqual(checkDraft(draft), {
    ok: true,
    input: { ...saved, disk_gb: 40 },
  });
});

test("a size needs whole numbers of at least 1", () => {
  const draft = newDraft([]);
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
  const started = newDraft([label("a", { name: "ciproxy", vcpus: 2 })]);
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
