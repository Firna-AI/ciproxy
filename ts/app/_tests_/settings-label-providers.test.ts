import assert from "node:assert/strict";
import test from "node:test";

import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";

import type { ProviderHold } from "../src/api/schema.ts";
import { fitLine } from "../src/view/settings-label-fit.ts";
import { fit, labelInput, providerFit } from "./settings-support.ts";

const nbsp = "\u00a0";
const request = `8${nbsp}vCPU, 32${nbsp}GB of memory, and 60${nbsp}GB of free disk`;

const none = {
  fitting_machines: 0,
  minimum_jobs: null,
  maximum_jobs: null,
  no_machine_fits: true,
};

const heldBy = (hold: ProviderHold) => ({
  routing: {
    kind: "waits",
    reason: { kind: "provider", provider: "boat", hold },
  } as const,
});

test("a placement that the API cannot establish keeps the known facts and names the unknown one", () => {
  const unknown = {
    fitting_places: null,
    providers: [providerFit({ fit: { state: "unavailable" } })],
  };
  assert.deepEqual(fitLine(labelInput(), fit(unknown)), {
    kind: "caption",
    text: "Each of your machines runs 3 jobs of this size at a time.",
  });
  assert.deepEqual(
    fitLine(
      labelInput(),
      fit({
        ...unknown,
        ...none,
        no_machine_fits: false,
        unavailable_machines: 3,
      }),
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
      text: "Your machines are not in the order of this label.",
    },
  );
  assert.deepEqual(fitLine(labelInput(), fit({ ...unknown, ...none })), {
    kind: "warning",
    title: `No machine of yours fits a job of ${request}`,
    body: "Boat sizes are not known yet.",
  });
  assert.deepEqual(
    fitLine(
      labelInput(),
      fit({
        ...unknown,
        ...none,
        fitting_places: [],
        providers: [
          providerFit({ fit: { state: "unavailable" }, connected: null }),
        ],
      }),
    ),
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Boat sizes are not known yet.",
    },
    "a wait never says that no Boat size fits before the catalog does",
  );
});

test("a provider that cannot take jobs gives its reason, and never that its sizes do not fit", () => {
  const notConnected = {
    providers: [
      providerFit({
        connected: false,
        fit: { state: "unavailable" },
        hold: "not_connected",
        largest_size: null,
      }),
    ],
  };
  const lines = [
    fitLine(
      labelInput(),
      fit({ ...notConnected, fitting_places: [machinePlace] }),
    ),
    fitLine(
      labelInput(),
      fit({
        ...notConnected,
        ...none,
        fitting_places: [],
        ...heldBy("not_connected"),
      }),
    ),
    fitLine(
      labelInput(),
      fit({ ...none, fitting_places: [boatPlace], ...heldBy("spend_limit") }),
    ),
    fitLine(
      labelInput(),
      fit({ ...none, fitting_places: [boatPlace], ...heldBy("off") }),
    ),
  ];
  assert.deepEqual(lines, [
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time.",
    },
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Boat is not connected, so jobs with this label wait.",
    },
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Boat reached its spend limit, so jobs with this label wait.",
    },
    {
      kind: "warning",
      title: `No machine of yours fits a job of ${request}`,
      body: "Overflow providers are off, so jobs with this label wait.",
    },
  ]);
  assert.equal(
    JSON.stringify(lines.slice(0, 2)).includes("Boat size"),
    false,
    "without a catalog, no line says that a Boat size does not fit",
  );
  assert.deepEqual(
    fitLine(
      labelInput({ order: [machinePlace] }),
      fit({
        ...notConnected,
        providers: [providerFit({ fit: { state: "disabled" } })],
        fitting_places: [machinePlace],
      }),
    ),
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time.",
    },
    "a label without a provider does not mention the account",
  );
});
