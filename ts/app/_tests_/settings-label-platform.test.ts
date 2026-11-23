import assert from "node:assert/strict";
import test from "node:test";

import { machinePlace } from "../scripts/m20-fixtures-labels.ts";

import { parseSettings } from "../src/api/settings-parse.ts";
import {
  checkDraft,
  diskLimitError,
  editDraft,
  newDraft,
} from "../src/view/settings-label-form.ts";
import { fitLine } from "../src/view/settings-label-fit.ts";
import { labelRows } from "../src/view/settings-labels.ts";
import { fit, label, labelInput, settingsView } from "./settings-support.ts";

const nbsp = " ";
const mac = { platform: "macos_arm64", name: "ciproxy-macos" } as const;

test("a new label is Linux, and an edit keeps the saved platform", () => {
  assert.equal(newDraft([]).platform, "linux_x64");
  assert.equal(editDraft(labelInput()).platform, "linux_x64");
  assert.equal(editDraft(labelInput(mac)).platform, "macos_arm64");
});

test("a macOS label always goes to the API with Boat off and Wait", () => {
  const draft = {
    ...editDraft(labelInput(mac)),
    allowBoat: true,
    allowMachines: false,
    whenNothingFits: "largest_overflow" as const,
  };
  const check = checkDraft(draft);
  assert.ok(check.ok);
  assert.deepEqual(
    [check.input.platform, check.input.order, check.input.when_nothing_fits],
    ["macos_arm64", [machinePlace], "wait"],
  );
});

test("the free disk keeps a most value when the API gives one", () => {
  const draft = { ...editDraft(labelInput(mac)), disk: "90" };
  assert.ok(checkDraft(draft).ok);
  assert.deepEqual(checkDraft(draft, { maxDiskGb: 80 }), {
    ok: false,
    errors: { disk: diskLimitError(80) },
  });
  assert.ok(checkDraft(draft, { maxDiskGb: 90 }).ok);
});

test("the fit line of a macOS label names Macs and never Boat", () => {
  const input = labelInput({ ...mac, order: [machinePlace] });
  assert.deepEqual(
    fitLine(
      input,
      fit({
        fitting_places: [machinePlace],
        eligible_machines: 1,
        fitting_machines: 1,
        minimum_jobs: "2",
        maximum_jobs: "2",
      }),
    ),
    {
      kind: "caption",
      text: "Each of your Macs runs 2 jobs of this size at a time.",
    },
  );
  const none = fitLine(
    input,
    fit({
      fitting_places: [],
      eligible_machines: 0,
      fitting_machines: 0,
      minimum_jobs: null,
      maximum_jobs: null,
    }),
  );
  assert.equal(none.kind, "warning");
  assert.match(none.kind === "warning" ? none.title : "", /^No Mac of yours/);
});

test("each label row names its platform, and a macOS label runs on Macs", () => {
  const rows = labelRows([
    label("1"),
    label(
      "2",
      { ...mac, order: [machinePlace] },
      { fitting_places: [machinePlace] },
    ),
  ]);
  assert.deepEqual(
    rows.map((row) => [row.size.split(`${nbsp}· `)[0], row.where.text]),
    [
      ["Linux", "Your machines, then Boat"],
      ["macOS", "Your Macs only"],
    ],
  );
});

test("the settings reader keeps the platform of a label, absent as absent", () => {
  const view = settingsView({
    labels: [label("1"), label("2", { ...mac, order: [machinePlace] })],
  });
  const read = parseSettings(JSON.parse(JSON.stringify(view)));
  assert.deepEqual(
    read?.labels.map((item) => item.label.platform),
    [undefined, "macos_arm64"],
  );
});
