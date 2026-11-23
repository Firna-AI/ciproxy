import assert from "node:assert/strict";
import test from "node:test";

import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";

import { fitLine } from "../src/view/settings-label-fit.ts";
import { fit, labelInput, providerFit } from "./settings-support.ts";

const nbsp = "\u00a0";
const request = `8${nbsp}vCPU, 32${nbsp}GB of memory, and 60${nbsp}GB of free disk`;

test("a placement that the API cannot establish keeps the known facts and names the unknown one", () => {
  const unknown = {
    fitting_places: null,
    providers: [providerFit({ fit: { state: "unavailable" } })],
  };
  const none = { fitting_machines: 0, minimum_jobs: null, maximum_jobs: null };
  assert.deepEqual(fitLine(labelInput(), fit(unknown)), {
    kind: "caption",
    text: "Each of your machines runs 3 jobs of this size at a time. Its Boat fit is not known yet.",
  });
  assert.deepEqual(
    fitLine(
      labelInput(),
      fit({ ...unknown, ...none, unavailable_machines: 3 }),
    ),
    {
      kind: "caption",
      text: "The capacity of your machines is not known yet.",
    },
  );
  assert.deepEqual(
    fitLine(labelInput({ order: [boatPlace] }), fit({ ...unknown, ...none })),
    {
      kind: "caption",
      text: "Only Boat may run jobs with this label. Its Boat fit is not known yet.",
    },
  );
  assert.deepEqual(
    fitLine(labelInput(), fit({ ...unknown, ...none, no_machine_fits: true })),
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Its Boat fit is not known yet.",
    },
  );
  assert.deepEqual(
    fitLine(
      labelInput(),
      fit({
        ...unknown,
        providers: [
          providerFit({ fit: { state: "unavailable" }, connected: null }),
        ],
      }),
    ),
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time. Its Boat fit is not known yet.",
    },
    "an account that ciproxy could not read is unknown, not missing",
  );
  assert.deepEqual(
    fitLine(
      labelInput(),
      fit({
        ...unknown,
        ...none,
        no_machine_fits: true,
        fitting_places: [],
        providers: [
          providerFit({ fit: { state: "unavailable" }, connected: null }),
        ],
      }),
    ),
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Its Boat fit is not known yet.",
    },
    "a wait never says that no Boat size fits before the catalog does",
  );
});

test("a workspace without Boat keeps the fit of the machines and says that Boat is not connected", () => {
  const noBoat = {
    providers: [
      providerFit({
        connected: false,
        fit: { state: "unavailable" },
        largest_size: null,
      }),
    ],
  };
  const none = {
    fitting_machines: 0,
    minimum_jobs: null,
    maximum_jobs: null,
    no_machine_fits: true,
  };
  const lines = [
    fitLine(labelInput(), fit({ ...noBoat, fitting_places: [machinePlace] })),
    fitLine(labelInput(), fit({ ...noBoat, ...none, fitting_places: [] })),
    fitLine(
      labelInput({ order: [boatPlace] }),
      fit({ ...noBoat, ...none, fitting_places: [] }),
    ),
  ];
  assert.deepEqual(lines, [
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time. Boat is not connected.",
    },
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Boat is not connected, so jobs with this label wait until a machine that fits connects.",
    },
    {
      kind: "warning",
      title: "Boat is not connected",
      body: "Only Boat may run jobs with this label, so they wait until you connect Boat.",
    },
  ]);
  assert.equal(
    JSON.stringify(lines).includes("Boat size"),
    false,
    "without a catalog, no line says that a Boat size does not fit",
  );
  assert.deepEqual(
    fitLine(
      labelInput({ order: [machinePlace] }),
      fit({
        ...noBoat,
        providers: [providerFit({ fit: { state: "disabled" } })],
        fitting_places: [machinePlace],
      }),
    ),
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time.",
    },
    "a label without Boat does not mention the account",
  );
});
