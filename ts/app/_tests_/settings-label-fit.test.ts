import assert from "node:assert/strict";
import test from "node:test";

import { boatPlace, machinePlace } from "../scripts/m20-fixtures-labels.ts";

import { fitLine } from "../src/view/settings-label-fit.ts";
import { vmDiskLine } from "../src/view/settings-label-sizes.ts";
import {
  fit,
  labelInput,
  offer,
  providerFit,
  xlarge,
} from "./settings-support.ts";

const nbsp = "\u00a0";
const request = `8${nbsp}vCPU, 32${nbsp}GB of memory, and 60${nbsp}GB of free disk`;
const largestBoat = "Jobs run on the largest size of your Boat account.";

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
        fitting_places: [machinePlace],
      }),
      line: "1 job of this size at a time on 3 of your 4 machines.",
    },
    {
      name: "the capacity of a machine is not known",
      fit: fit({
        eligible_machines: 3,
        fitting_machines: 2,
        minimum_jobs: "2",
        maximum_jobs: "2",
        unavailable_machines: 1,
      }),
      line: "2 jobs of this size at a time on 2 of your 3 machines. The capacity of 1 machine is not known yet.",
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
    string | null,
  ][] = [
    [
      {},
      { fitting_places: [boatPlace] },
      `No machine of yours fits a job of ${request}`,
      "Jobs with this label run on Boat.",
    ],
    [
      {},
      {
        fitting_places: [boatPlace],
        providers: [providerFit({ fit: fallback })],
      },
      `No machine of yours and no Boat size fits a job of ${request}`,
      largestBoat,
    ],
    [
      {},
      {
        fitting_places: [],
        providers: [providerFit({ fit: { state: "no_size_fits" } })],
      },
      `No machine of yours and no Boat size fits a job of ${request}`,
      null,
    ],
    [
      { order: [machinePlace] },
      {
        fitting_places: [],
        providers: [providerFit({ fit: { state: "disabled" } })],
      },
      `No machine of yours fits a job of ${request}`,
      "Jobs with this label wait until a machine that fits connects.",
    ],
    [
      { order: [machinePlace] },
      {
        fitting_places: [],
        providers: [providerFit({ fit: { state: "disabled" } })],
        eligible_machines: 0,
        machines: [],
      },
      `No machine of yours fits a job of ${request}`,
      "No connected machine can run jobs now. Jobs with this label wait until a machine that fits connects.",
    ],
    [
      { order: [boatPlace] },
      {
        fitting_places: [],
        providers: [providerFit({ fit: { state: "no_size_fits" } })],
      },
      `No Boat size fits a job of ${request}`,
      null,
    ],
    [
      { order: [] },
      {
        fitting_places: [],
        providers: [providerFit({ fit: { state: "disabled" } })],
      },
      "Turn on Your machines or Boat",
      "Jobs with this label wait until one of them is on.",
    ],
    [
      {},
      {
        fitting_places: [boatPlace],
        machines: [
          offer("1", { slots: "6", vcpus: "12", memory_bytes: "48000000000" }),
          offer("2"),
        ],
      },
      `No machine of yours fits a job of ${request}`,
      "Jobs with this label run on Boat.",
    ],
    [
      {},
      {
        fitting_places: [boatPlace],
        eligible_machines: 1,
        machines: [offer("1")],
      },
      `No machine of yours fits a job of ${request}`,
      "Jobs with this label run on Boat.",
    ],
    [
      {},
      { fitting_places: [boatPlace], eligible_machines: 0, machines: [] },
      `No machine of yours fits a job of ${request}`,
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
    line({ order: [boatPlace] }, { fitting_places: [boatPlace] }),
    {
      kind: "caption",
      text: "Only Boat may run jobs with this label.",
    },
  );
});

test("the dialog names the VM disk of the size, and never a size of the Boat account", () => {
  assert.equal(
    vmDiskLine(fit()),
    `With the current image, each VM has a disk of 122${nbsp}GB.`,
  );
  assert.equal(
    vmDiskLine(
      fit({ providers: [providerFit({ required_root_disk_bytes: null })] }),
    ),
    null,
  );
  const fallback = fit({
    fitting_places: [boatPlace],
    fitting_machines: 0,
    minimum_jobs: null,
    maximum_jobs: null,
    no_machine_fits: true,
    providers: [
      providerFit({
        fit: {
          state: "largest_fallback",
          refused: false,
          size: xlarge.size,
          size_name: xlarge.name,
        },
      }),
    ],
  });
  const nothingFits = {
    kind: "warning",
    title: `No machine of yours and no Boat size fits a job of ${request}`,
    body: largestBoat,
  };
  assert.deepEqual(fitLine(labelInput(), fallback), nothingFits);
  assert.deepEqual(
    fitLine(labelInput(), {
      ...fallback,
      providers: fallback.providers.map((provider) => ({
        ...provider,
        largest_size: null,
      })),
    }),
    nothingFits,
  );
  assert.deepEqual(fitLine(labelInput({ order: [boatPlace] }), fallback), {
    kind: "caption",
    text: `Only Boat may run jobs with this label. ${largestBoat}`,
  });
});
