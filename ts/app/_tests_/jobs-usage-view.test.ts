import assert from "node:assert/strict";
import test from "node:test";

import {
  figureLine,
  usagePanelModel,
  type MeasuredUsage,
} from "../src/view/jobs-usage.ts";
import { fixtureJob } from "../scripts/fixtures-jobs.ts";
import { figures, measured, plain, step } from "./jobs-usage-support.ts";

function figureRows(usage: MeasuredUsage) {
  const model = usagePanelModel(fixtureJob(), usage);
  assert.ok(model?.state === "measured");
  return model.figures.map((figure) => [
    figure.label,
    plain(figure.value),
    plain(figure.caption),
    figure.meter,
    figure.alert,
  ]);
}

test("the figures of a job read against the VM, with the label size where the VM is larger and the kills only on Linux", () => {
  assert.deepEqual(figureRows(measured()), [
    ["Average CPU", "1.6 vCPU", "of 2 vCPU", { share: 0.8, mark: null }, false],
    ["Peak memory", "6.1 GB", "of 8 GB", { share: 0.7625, mark: null }, false],
    ["Out of memory", "None", null, null, false],
  ]);
  const boat = measured({
    job: figures(4400, "7300000000", 2),
    vm: { vcpus: 8, memory_bytes: "16000000000" },
    label_mark: { vcpus: 2, memory_bytes: "8000000000" },
  });
  assert.deepEqual(figureRows(boat), [
    [
      "Average CPU",
      "4.4 vCPU",
      "of 8 vCPU · label 2 vCPU",
      { share: 0.55, mark: 0.25 },
      false,
    ],
    [
      "Peak memory",
      "7.3 GB",
      "of 16 GB · label 8 GB",
      { share: 0.45625, mark: 0.5 },
      false,
    ],
    ["Out of memory", "2 processes", null, null, true],
  ]);
  const macos = measured({
    job: figures(1500, "6900000000", null),
    label_mark: { vcpus: null, memory_bytes: "4000000000" },
  });
  assert.deepEqual(figureRows(macos), [
    [
      "Average CPU",
      "1.5 vCPU",
      "of 2 vCPU",
      { share: 0.75, mark: null },
      false,
    ],
    [
      "Peak memory",
      "6.9 GB",
      "of 8 GB · label 4 GB",
      { share: 0.8625, mark: 0.5 },
      false,
    ],
  ]);
});

test("each step shows its duration and its figures, and a name that hides a script links to its step on GitHub", () => {
  const steps = [
    step(1, "Set up job", 0, 2, { figures: figures(300, "500000000") }),
    step(2, "Run script", 2, 11, {
      script: true,
      figures: figures(500, "1400000000"),
    }),
    step(3, "Compare screenshots", 11, 102, {
      figures: figures(1600, "7800000000", 1),
    }),
    step(4, "Post Run actions/setup-node@v4", null, null, { skipped: true }),
    step(5, "Upload diff images", 102, null),
    step(6, "Post Run actions/checkout@v4", 105, 105),
    step(7, "Complete job", 105, 106, { figures: figures(200, "1200000000") }),
  ];
  const model = usagePanelModel(fixtureJob(), measured({ steps }));
  assert.ok(model?.state === "measured");
  assert.deepEqual(
    model.steps?.map((row) => [
      row.number,
      row.name,
      row.href,
      plain(row.duration),
      row.timed,
      row.figures === null
        ? null
        : [
            plain(row.figures.cpu),
            plain(row.figures.memory),
            row.figures.kills,
          ],
    ]),
    [
      [1, "Set up job", null, "2s", true, ["0.3 vCPU", "0.5 GB", null]],
      [
        2,
        "Run script",
        "https://github.com/northstar-labs/control/actions/runs/701/job/801#step:2:1",
        "9s",
        true,
        ["0.5 vCPU", "1.4 GB", null],
      ],
      [
        3,
        "Compare screenshots",
        null,
        "1m 31s",
        true,
        ["1.6 vCPU", "7.8 GB", "1 process"],
      ],
      [4, "Post Run actions/setup-node@v4", null, "Skipped", false, null],
      [5, "Upload diff images", null, "Unknown", false, null],
      [6, "Post Run actions/checkout@v4", null, "0s", true, null],
      [7, "Complete job", null, "1s", true, ["0.2 vCPU", "1.2 GB", null]],
    ],
  );
  assert.equal(model.killColumn, true);
  const second = model.steps?.[1]?.figures ?? null;
  assert.equal(
    plain(second === null ? null : figureLine(second)),
    "0.5 vCPU average · 1.4 GB peak",
  );
  const quiet = usagePanelModel(
    fixtureJob({ github_url: null }),
    measured({ steps: steps.slice(0, 2) }),
  );
  assert.ok(quiet?.state === "measured");
  assert.deepEqual(
    [quiet.steps?.[1]?.href, quiet.killColumn],
    [null, false],
    "without a GitHub page, a script step has no link",
  );
  const noSteps = usagePanelModel(fixtureJob(), measured());
  assert.ok(noSteps?.state === "measured");
  assert.equal(noSteps.steps, null);
});

test("a running job and a job without figures say so, and a job that no runner took has no panel", () => {
  assert.deepEqual(usagePanelModel(fixtureJob(), { state: "running" }), {
    state: "running",
    text: "The figures show when the job ends.",
  });
  assert.deepEqual(usagePanelModel(fixtureJob(), { state: "unavailable" }), {
    state: "unavailable",
    text: "Figures are not available for this job.",
  });
  const waiting = fixtureJob({
    status: "queued",
    location: null,
    counts_as_start: false,
  });
  assert.equal(usagePanelModel(waiting, { state: "unavailable" }), null);
  const unknownPlace = fixtureJob({ location: null, counts_as_start: true });
  assert.equal(
    usagePanelModel(unknownPlace, { state: "unavailable" })?.state,
    "unavailable",
  );
});
