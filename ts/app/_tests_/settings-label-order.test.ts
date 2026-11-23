import assert from "node:assert/strict";
import test from "node:test";

import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";

import type { LabelFit, ProviderLabelFit } from "../src/api/schema.ts";
import {
  checkDraft,
  createRequest,
  editDraft,
  newDraft,
  withOrder,
} from "../src/view/settings-label-form.ts";
import { placeCaption, placeFacts } from "../src/view/settings-label-order.ts";
import { addablePlaces } from "../src/view/settings-label-places.ts";
import { fit, labelInput, providerFit, xlarge } from "./settings-support.ts";

const input = labelInput();

/** The words under each place of the order of `input` for one fit. */
function captions(
  labelFit: LabelFit,
  waitSeconds: number | null = 30,
  label = input,
) {
  return label.order.map((place, index) =>
    placeCaption(label, place, index, labelFit, waitSeconds),
  );
}

function boatRow(overrides: Partial<ProviderLabelFit>) {
  return captions(fit({ providers: [providerFit(overrides)] }))[1];
}

test("a new Linux label starts with own machines, then each offered provider, and leaves that order to the API until it changes", () => {
  assert.deepEqual(newDraft([], ["boat"]).order, [machinePlace, boatPlace]);
  assert.deepEqual(newDraft([], []).order, [machinePlace]);
  const draft = {
    ...newDraft([], ["boat"]),
    name: "builds",
    vcpus: "2",
    memory: "8",
    disk: "20",
  };
  const check = checkDraft(draft);
  assert.ok(check.ok);
  assert.equal(createRequest(check.input, draft).order, undefined);
  const moved = withOrder(draft, [boatPlace, machinePlace]);
  const changed = checkDraft(moved);
  assert.ok(changed.ok);
  assert.deepEqual(createRequest(changed.input, moved).order, [
    boatPlace,
    machinePlace,
  ]);
  const mac = { ...draft, platform: "macos_arm64" as const };
  const macCheck = checkDraft(mac);
  assert.ok(macCheck.ok);
  assert.deepEqual(createRequest(macCheck.input, mac).order, [machinePlace]);
  assert.deepEqual(checkDraft(withOrder(editDraft(input), [])), {
    ok: true,
    input: { ...input, platform: "linux_x64", order: [] },
  });
});

test("Add a place offers own machines and each offered provider that the order does not hold", () => {
  assert.deepEqual(addablePlaces([machinePlace, boatPlace], ["boat"]), []);
  assert.deepEqual(addablePlaces([boatPlace], ["boat"]), [machinePlace]);
  assert.deepEqual(addablePlaces([], ["boat"]), [machinePlace, boatPlace]);
  assert.deepEqual(addablePlaces([machinePlace], []), []);
});

test("an order row of own machines gives the overflow wait only when a place follows them", () => {
  assert.deepEqual(captions(fit()), [
    "Jobs wait up to 30 seconds for a free slot",
    "Runs on xlarge, 16 vCPU and 32 GB",
  ]);
  assert.deepEqual(
    captions(fit(), 0)[0],
    "Jobs go to the next place at once when no slot is free",
  );
  assert.deepEqual(
    captions(fit(), null)[0],
    "Jobs wait up to the overflow wait for a free slot",
  );
  const last = labelInput({ order: [boatPlace, machinePlace] });
  assert.deepEqual(captions(fit(), 30, last)[1], "Jobs wait for a free slot");
  const none = { fitting_machines: 0, no_machine_fits: true };
  assert.deepEqual(
    [
      captions(fit(none))[0],
      captions(fit({ ...none, eligible_machines: 0 }))[0],
      captions(fit({ fitting_machines: 0, unavailable_machines: 2 }))[0],
    ],
    [
      "No machine of yours fits this size",
      "No machine of yours can run jobs now",
      "The capacity of your machines is not known yet",
    ],
  );
  const mac = labelInput({ platform: "macos_arm64", order: [machinePlace] });
  assert.deepEqual(captions(fit(), 30, mac), ["Jobs wait for a free Mac"]);
});

test("an order row of a provider says off, not connected, or unavailable before it says whether a size fits", () => {
  const unknownSizes = { state: "unavailable" } as const;
  const noSize = { state: "no_size_fits" } as const;
  assert.deepEqual(
    [
      boatRow({ hold: "off", fit: noSize }),
      boatRow({ hold: "not_connected", fit: unknownSizes, connected: false }),
      boatRow({ hold: "unavailable", fit: unknownSizes }),
      boatRow({ fit: noSize }),
      boatRow({ hold: "spend_limit" }),
      boatRow({ hold: "size_refused", eligible_size: null }),
      boatRow({ fit: unknownSizes }),
      boatRow({
        fit: {
          state: "largest_fallback",
          refused: false,
          size: xlarge.size,
          size_name: xlarge.name,
        },
      }),
    ],
    [
      "Overflow providers are off, so jobs skip it",
      "Boat is not connected, so jobs skip it",
      "Boat is unavailable right now, so jobs skip it",
      "No Boat size fits, so jobs skip it",
      "Boat reached its spend limit, so jobs skip it",
      "Boat refused this size, so jobs skip it",
      "Boat sizes are not known yet",
      "Runs on its largest size, xlarge, 16 vCPU and 32 GB",
    ],
  );
});

test("a move of a place keeps the words of each row, and a new size or a new place waits for its fit", () => {
  const shown = { input, fit: fit() };
  const moved = { ...input, order: [boatPlace, machinePlace] };
  assert.equal(placeFacts(moved, shown), shown.fit);
  assert.equal(placeFacts({ ...input, vcpus: 16 }, shown), null);
  assert.equal(placeFacts({ ...input, order: [machinePlace] }, shown), null);
  assert.equal(placeFacts(input, null), null);
  assert.equal(placeCaption(input, machinePlace, 0, null, 30), null);
});
