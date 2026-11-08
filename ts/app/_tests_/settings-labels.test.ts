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
import { canDelete, fitLine, labelRows } from "../src/view/settings-labels.ts";
import { fit, label, labelInput } from "./settings-support.ts";

test("the fit line gives one count, a range, or the machines that fit", () => {
  const size = labelInput();
  const cases = [
    {
      name: "every machine fits the same count",
      fit: fit({ minimum_jobs: "3", maximum_jobs: "3" }),
      line: "Each of your machines runs 3 jobs of this size at a time.",
    },
    {
      name: "machines fit different counts",
      fit: fit({ minimum_jobs: "1", maximum_jobs: "3" }),
      line: "Each of your machines runs 1 to 3 jobs of this size at a time.",
    },
    {
      name: "some machines fit none",
      fit: fit({
        eligible_machines: 4,
        fitting_machines: 3,
        minimum_jobs: "1",
        maximum_jobs: "1",
      }),
      line: "1 job of this size at a time on 3 of your 4 machines.",
    },
    {
      name: "a machine has not reported its capacity",
      fit: fit({
        eligible_machines: 3,
        fitting_machines: 2,
        minimum_jobs: "2",
        maximum_jobs: "2",
        unavailable_machines: 1,
      }),
      line: "2 jobs of this size at a time on 2 of your 3 machines. 1 machine has not reported its capacity yet.",
    },
    {
      name: "no machine can run jobs",
      fit: fit({
        eligible_machines: 0,
        fitting_machines: 0,
        minimum_jobs: null,
        maximum_jobs: null,
        no_machine_fits: true,
      }),
      line: "No connected machine can run jobs now.",
    },
  ];
  for (const item of cases) {
    assert.deepEqual(
      fitLine(size, item.fit),
      { kind: "caption", text: item.line },
      item.name,
    );
  }
});

test("a size that fits no machine says before the save where its jobs go", () => {
  const none = fit({
    fitting_machines: 0,
    minimum_jobs: null,
    maximum_jobs: null,
    no_machine_fits: true,
  });
  assert.deepEqual(fitLine(labelInput({ allow_boat: true }), none), {
    kind: "warning",
    title: "No machine fits this size",
    body: "Only Boat may run jobs with this label. Its Boat fit is not available.",
  });
  assert.deepEqual(fitLine(labelInput({ allow_boat: false }), none), {
    kind: "warning",
    title: "No machine fits this size",
    body: "Jobs with this label wait until a machine that fits connects.",
  });
  assert.deepEqual(
    fitLine(labelInput({ allow_machines: false, allow_boat: true }), fit()),
    {
      kind: "caption",
      text: "Only Boat may run jobs with this label. Its Boat fit is not available.",
    },
  );
});

test("the label list states the machine fit and only the Boat policy", () => {
  const none = { fitting_machines: 0, no_machine_fits: true };
  const boat = "Boat allowed\u00a0· Boat fit not available";
  const rows = labelRows([
    label("a", { name: "ciproxy" }),
    label("b", { name: "large", allow_boat: false }),
    label("c", { name: "huge" }, none),
    label("d", { name: "huge-own", allow_boat: false }, none),
    label(
      "e",
      { name: "new-box" },
      { fitting_machines: 0, unavailable_machines: 1 },
    ),
    label("f", { name: "boat-only", allow_machines: false }),
    label("g", { name: "off", allow_machines: false, allow_boat: false }),
  ]);
  assert.deepEqual(
    rows.map((row) => [row.name, row.isDefault, row.where]),
    [
      ["ciproxy", true, { text: "Your machines", warn: false, boat }],
      ["large", false, { text: "Your machines", warn: false, boat: null }],
      ["huge", false, { text: "No machine fits", warn: false, boat }],
      [
        "huge-own",
        false,
        { text: "Waits: no machine fits", warn: true, boat: null },
      ],
      [
        "new-box",
        false,
        { text: "Machine capacity not known yet", warn: false, boat },
      ],
      ["boat-only", false, { text: "Your machines off", warn: false, boat }],
      [
        "off",
        false,
        {
          text: "Waits: Your machines and Boat are off",
          warn: true,
          boat: null,
        },
      ],
    ],
  );
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
