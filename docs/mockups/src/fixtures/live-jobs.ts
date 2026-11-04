// Running and waiting jobs of mokly-ai at the moments the live queue shows.

import { none, onBoat, own, rows } from "./jobs.ts";

/** Wednesday 18 November, 14:32. 21 running; the first 10 are listed. */
export const steadyRunning = rows("steady", [
  ["test (rust)", "CI", "mokly", "#1542", "running", own(1, 1), "4m 12s"],
  ["clippy", "CI", "mokly", "#1542", "running", own(1, 2), "2m 05s"],
  [
    "e2e (chromium) 1/4",
    "E2E",
    "mokly",
    "main",
    "running",
    own(2, 1),
    "6m 40s",
  ],
  [
    "e2e (chromium) 2/4",
    "E2E",
    "mokly",
    "main",
    "running",
    own(2, 2),
    "6m 38s",
  ],
  ["e2e (chromium) 3/4", "E2E", "mokly", "main", "running", onBoat, "6m 31s"],
  ["e2e (chromium) 4/4", "E2E", "mokly", "main", "running", onBoat, "6m 30s"],
  [
    "build (linux-x64)",
    "Release",
    "mokly-cloud",
    "#318",
    "running",
    own(3, 1),
    "1m 48s",
  ],
  [
    "unit (node 24)",
    "CI",
    "mokly-cloud",
    "#318",
    "running",
    own(3, 2),
    "1m 10s",
  ],
  ["visual diff", "CI", "mokly", "#1539", "running", own(1, 3), "52s"],
  ["typecheck", "CI", "mokly", "#1539", "running", own(2, 3), "41s"],
]);

/** Wednesday 18 November, 10:05, a release peak. 12 wait, 64 run. */
export const peakWaiting = rows("peak-wait", [
  [
    "e2e (webkit) 1/2",
    "E2E",
    "mokly",
    "#1544",
    "waiting",
    none,
    "24s",
    "ciproxy-4vcpu",
  ],
  [
    "e2e (webkit) 2/2",
    "E2E",
    "mokly",
    "#1544",
    "waiting",
    none,
    "24s",
    "ciproxy-4vcpu",
  ],
  ["storybook", "CI", "mokly", "#1544", "waiting", none, "19s"],
  ["unit (node 24)", "CI", "mokly-cloud", "#320", "waiting", none, "15s"],
  [
    "docker image",
    "Release",
    "mokly-cloud",
    "main",
    "waiting",
    none,
    "9s",
    "ciproxy-8vcpu",
  ],
  ["fmt", "CI", "mokly", "#1545", "waiting", none, "4s"],
]);

export const peakRunning = rows("peak-run", [
  [
    "build (linux-x64)",
    "Release",
    "mokly",
    "v0.19.0",
    "running",
    own(1, 1),
    "3m 02s",
  ],
  [
    "build (linux-arm64)",
    "Release",
    "mokly",
    "v0.19.0",
    "running",
    own(2, 1),
    "3m 01s",
  ],
  ["e2e (chromium) 1/4", "E2E", "mokly", "#1544", "running", onBoat, "1m 12s"],
  ["e2e (chromium) 2/4", "E2E", "mokly", "#1544", "running", onBoat, "1m 12s"],
  ["test (rust)", "CI", "mokly", "#1544", "running", own(3, 1), "2m 40s"],
  ["clippy", "CI", "mokly", "#1544", "running", onBoat, "58s"],
  ["unit (node 24)", "CI", "mokly-cloud", "#319", "running", onBoat, "44s"],
  ["typecheck", "CI", "mokly-cloud", "#319", "running", own(3, 4), "39s"],
]);

/** Thursday 26 November, 11:40: Boat reached its spend limit. */
export const limitWaiting = rows("limit-wait", [
  ["e2e (chromium) 3/4", "E2E", "mokly", "#1561", "waiting", none, "4m 12s"],
  ["e2e (chromium) 4/4", "E2E", "mokly", "#1561", "waiting", none, "4m 12s"],
  ["visual diff", "CI", "mokly", "#1561", "waiting", none, "3m 50s"],
  ["unit (node 24)", "CI", "mokly-cloud", "#327", "waiting", none, "2m 31s"],
  ["typecheck", "CI", "mokly-cloud", "#327", "waiting", none, "2m 30s"],
  ["storybook", "CI", "mokly", "#1562", "waiting", none, "1m 05s"],
]);

export const limitRunning = rows("limit-run", [
  ["test (rust)", "CI", "mokly", "#1561", "running", own(1, 1), "5m 20s"],
  [
    "e2e (chromium) 1/4",
    "E2E",
    "mokly",
    "#1561",
    "running",
    own(2, 1),
    "4m 58s",
  ],
  [
    "e2e (chromium) 2/4",
    "E2E",
    "mokly",
    "#1561",
    "running",
    own(2, 2),
    "4m 58s",
  ],
  [
    "build (linux-x64)",
    "Release",
    "mokly-cloud",
    "#327",
    "running",
    own(3, 1),
    "2m 14s",
  ],
]);

/** Friday 4 September, 15:10, on Free with one machine. */
export const freeRunning = rows("free-run", [
  ["test (rust)", "CI", "mokly", "#1497", "running", own(1, 1), "3m 41s"],
  ["clippy", "CI", "mokly", "#1497", "running", own(1, 2), "1m 52s"],
  [
    "e2e (chromium) 1/4",
    "E2E",
    "mokly",
    "#1497",
    "running",
    own(1, 3),
    "1m 20s",
  ],
  [
    "e2e (chromium) 2/4",
    "E2E",
    "mokly",
    "#1497",
    "running",
    own(1, 4),
    "1m 20s",
  ],
  ["e2e (chromium) 3/4", "E2E", "mokly", "#1497", "running", onBoat, "1m 03s"],
  ["unit (node 24)", "CI", "mokly-cloud", "#301", "running", own(1, 5), "48s"],
]);
