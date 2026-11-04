// Jobs of mokly-ai that wait longer than the overflow wait: while Boat refuses
// new sandboxes, and while no machine and no Boat size fits their label.

import { none, onBoat, own, rows } from "./jobs.ts";

/** Wednesday 25 November, 10:12: the start limit of the Boat plan is full. */
export const boatRefused = {
  /** The time when Boat takes new sandboxes again. */
  retryAt: "10:13",
  waiting: rows("refused-wait", [
    ["e2e (webkit) 1/2", "E2E", "mokly", "#1571", "waiting", none, "1m 42s"],
    ["e2e (webkit) 2/2", "E2E", "mokly", "#1571", "waiting", none, "1m 42s"],
    ["storybook", "CI", "mokly", "#1571", "waiting", none, "1m 20s"],
    ["unit (node 24)", "CI", "mokly-cloud", "#331", "waiting", none, "58s"],
    ["typecheck", "CI", "mokly-cloud", "#331", "waiting", none, "51s"],
    ["fmt", "CI", "mokly", "#1572", "waiting", none, "37s"],
  ]),
  running: rows("refused-run", [
    ["test (rust)", "CI", "mokly", "#1571", "running", own(1, 1), "3m 15s"],
    ["clippy", "CI", "mokly", "#1571", "running", own(1, 2), "2m 48s"],
    [
      "e2e (chromium) 1/4",
      "E2E",
      "mokly",
      "#1571",
      "running",
      onBoat,
      "2m 02s",
    ],
    [
      "e2e (chromium) 2/4",
      "E2E",
      "mokly",
      "#1571",
      "running",
      onBoat,
      "2m 02s",
    ],
    [
      "build (linux-x64)",
      "Release",
      "mokly",
      "main",
      "running",
      own(2, 1),
      "1m 37s",
    ],
    ["visual diff", "CI", "mokly", "#1570", "running", onBoat, "1m 30s"],
    [
      "unit (node 24)",
      "CI",
      "mokly-cloud",
      "#330",
      "running",
      own(3, 1),
      "1m 05s",
    ],
    [
      "docker image",
      "Release",
      "mokly-cloud",
      "main",
      "running",
      onBoat,
      "48s",
    ],
  ]),
};

/** Thursday 19 November, 15:40: the load tests wait for ciproxy-32vcpu. */
export const nothingFits = {
  waiting: rows("fits-wait", [
    [
      "load test (api)",
      "Load",
      "mokly-cloud",
      "main",
      "waiting",
      none,
      "41m 10s",
    ],
    [
      "load test (viewer)",
      "Load",
      "mokly-cloud",
      "main",
      "waiting",
      none,
      "41m 10s",
    ],
  ]),
};
