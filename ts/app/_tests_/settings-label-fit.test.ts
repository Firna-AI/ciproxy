import assert from "node:assert/strict";
import test from "node:test";

import { fitLine } from "../src/view/settings-label-fit.ts";
import {
  largestBoatBody,
  vmDiskLine,
} from "../src/view/settings-label-sizes.ts";
import { fit, labelInput, offer, xlarge } from "./settings-support.ts";

const nbsp = "\u00a0";
const size = `8${nbsp}vCPU and 32${nbsp}GB`;
const largest = `16${nbsp}vCPU and 32${nbsp}GB`;
const machineOffer = `24${nbsp}vCPU and 96${nbsp}GB`;

test("the fit line gives one count, a range, or the machines that fit", () => {
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
        placement: "machines_only",
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
  ];
  for (const item of cases) {
    assert.deepEqual(
      fitLine(labelInput(), item.fit),
      { kind: "caption", text: item.line },
      item.name,
    );
  }
});

test("a size that fits no machine says where its jobs go: Boat, the largest Boat size, or a wait", () => {
  const none = {
    fitting_machines: 0,
    minimum_jobs: null,
    maximum_jobs: null,
    no_machine_fits: true,
  };
  const line = (
    input: Parameters<typeof labelInput>[0],
    overrides: Parameters<typeof fit>[0],
  ) => fitLine(labelInput(input), fit({ ...none, ...overrides }));
  const fallback = {
    state: "largest_fallback",
    refused: false,
    size: xlarge.size,
    size_name: xlarge.name,
  } as const;
  const cases: [
    Parameters<typeof labelInput>[0],
    Parameters<typeof fit>[0],
    string,
    string,
  ][] = [
    [
      {},
      { placement: "boat_only" },
      `No machine fits ${size}`,
      `Jobs with this label run on Boat. Your machines offer ${machineOffer} each.`,
    ],
    [
      {},
      { placement: "boat_only", boat: fallback },
      `No machine and no Boat size fits ${size}`,
      `Jobs run on ${largest}, the largest size of your Boat account.`,
    ],
    [
      {},
      { placement: "waits", boat: { state: "no_size_fits" } },
      `No machine and no Boat size fits ${size}`,
      `Your machines offer ${machineOffer} each, and the largest Boat size offers ${largest}.`,
    ],
    [
      { allow_boat: false },
      { placement: "waits", boat: { state: "disabled" } },
      `No machine fits ${size}`,
      `Your machines offer ${machineOffer} each. Jobs with this label wait until a machine that fits connects.`,
    ],
    [
      { allow_machines: false },
      { placement: "waits", boat: { state: "no_size_fits" } },
      `No Boat size fits ${size}`,
      `The largest Boat size offers ${largest}.`,
    ],
    [
      { allow_machines: false, allow_boat: false },
      { placement: "waits", boat: { state: "disabled" } },
      "Turn on Your machines or Boat",
      "Jobs with this label wait until one of them is on.",
    ],
    [
      {},
      {
        placement: "boat_only",
        machines: [
          offer("1", { slots: "6", vcpus: "12", memory_bytes: "48000000000" }),
          offer("2"),
        ],
      },
      `No machine fits ${size}`,
      `Jobs with this label run on Boat. Your largest machine offers ${machineOffer}.`,
    ],
    [
      {},
      { placement: "boat_only", eligible_machines: 1, machines: [offer("1")] },
      `No machine fits ${size}`,
      `Jobs with this label run on Boat. Your machine offers ${machineOffer}.`,
    ],
    [
      {},
      { placement: "boat_only", eligible_machines: 0, machines: [] },
      `No machine fits ${size}`,
      "Jobs with this label run on Boat.",
    ],
  ];
  for (const [input, overrides, title, body] of cases) {
    assert.deepEqual(
      line(input, overrides),
      { kind: "warning", title, body },
      title,
    );
  }
  assert.deepEqual(
    line({ allow_machines: false }, { placement: "boat_only" }),
    {
      kind: "caption",
      text: "Only Boat may run jobs with this label.",
    },
  );
});

test("the dialog names the VM disk of the size and the largest Boat size of the account", () => {
  assert.equal(
    vmDiskLine(fit()),
    `With the current image, each VM has a disk of 122${nbsp}GB.`,
  );
  assert.equal(vmDiskLine(fit({ boat_required_root_disk_bytes: null })), null);
  assert.equal(
    largestBoatBody(fit()),
    `Jobs run on ${largest}, the largest size of your Boat account.`,
  );
  const unknown =
    "Jobs run on the largest size of your Boat account, also when it is smaller than the label.";
  assert.equal(largestBoatBody(fit({ largest_boat: null })), unknown);
  assert.equal(largestBoatBody(null), unknown);
});
