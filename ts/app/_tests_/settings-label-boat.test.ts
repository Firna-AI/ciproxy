import assert from "node:assert/strict";
import test from "node:test";

import { fitLine } from "../src/view/settings-label-fit.ts";
import { largestBoatBody } from "../src/view/settings-label-sizes.ts";
import { fit, labelInput } from "./settings-support.ts";

const nbsp = "\u00a0";
const size = `8${nbsp}vCPU and 32${nbsp}GB`;
const machineOffer = `24${nbsp}vCPU and 96${nbsp}GB`;

test("a placement that the API cannot establish keeps the known facts and names the unknown one", () => {
  const unknown = {
    placement: "unavailable",
    boat: { state: "unavailable" },
  } as const;
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
      text: "Your machines have not reported their capacity yet.",
    },
  );
  assert.deepEqual(
    fitLine(
      labelInput({ allow_machines: false }),
      fit({ ...unknown, ...none }),
    ),
    {
      kind: "caption",
      text: "Only Boat may run jobs with this label. Its Boat fit is not known yet.",
    },
  );
  assert.deepEqual(
    fitLine(labelInput(), fit({ ...unknown, ...none, no_machine_fits: true })),
    {
      kind: "warning",
      title: `No machine fits ${size}`,
      body: "Its Boat fit is not known yet.",
    },
  );
  assert.deepEqual(
    fitLine(labelInput(), fit({ ...unknown, boat_connected: null })),
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
        placement: "waits",
        boat_connected: null,
      }),
    ),
    {
      kind: "warning",
      title: `No machine fits ${size}`,
      body: `Your machines offer ${machineOffer} each. Its Boat fit is not known yet.`,
    },
    "a wait never says that no Boat size fits before the catalog does",
  );
});

test("a workspace without Boat keeps the fit of the machines and says that Boat is not connected", () => {
  const noBoat = {
    boat: { state: "unavailable" },
    boat_connected: false,
    largest_boat: null,
  } as const;
  const none = {
    fitting_machines: 0,
    minimum_jobs: null,
    maximum_jobs: null,
    no_machine_fits: true,
  };
  const lines = [
    fitLine(labelInput(), fit({ ...noBoat, placement: "machines_only" })),
    fitLine(labelInput(), fit({ ...noBoat, ...none, placement: "waits" })),
    fitLine(
      labelInput({ allow_machines: false }),
      fit({ ...noBoat, ...none, placement: "waits" }),
    ),
  ];
  assert.deepEqual(lines, [
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time. Boat is not connected.",
    },
    {
      kind: "warning",
      title: `No machine fits ${size}`,
      body: `Your machines offer ${machineOffer} each. Boat is not connected, so jobs with this label wait until a machine that fits connects.`,
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
      labelInput({ allow_boat: false }),
      fit({
        ...noBoat,
        boat: { state: "disabled" },
        placement: "machines_only",
      }),
    ),
    {
      kind: "caption",
      text: "Each of your machines runs 3 jobs of this size at a time.",
    },
    "a label without Boat does not mention the account",
  );
  assert.equal(
    largestBoatBody(fit(noBoat)),
    "Jobs run on the largest size of your Boat account, also when it is smaller than the label.",
  );
});
