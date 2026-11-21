// Waiting jobs of mokly-ai for the wait reasons that come from current facts
// of the API: a label whose fit ciproxy cannot check yet, a label that starts
// on the largest Boat size, and a full queue while Boat cannot take jobs.

import { none, rows } from "./jobs.ts";
import { labelNamed, nothingFitsLabel } from "./settings.ts";

/** Monday 23 November, 09:12: a machine report is missing. */
export const unknownFit = {
  label: labelNamed("ciproxy-8vcpu"),
  waiting: rows("unknown-fit-wait", [
    [
      "integration (postgres)",
      "CI",
      "mokly-cloud",
      "#331",
      "waiting",
      none,
      "52s",
      "ciproxy-8vcpu",
    ],
    [
      "integration (redis)",
      "CI",
      "mokly-cloud",
      "#331",
      "waiting",
      none,
      "52s",
      "ciproxy-8vcpu",
    ],
    [
      "bench (catalogue)",
      "Bench",
      "mokly",
      "main",
      "waiting",
      none,
      "31s",
      "ciproxy-8vcpu",
    ],
  ]),
};

/**
 * Friday 27 November, 16:02: ciproxy-32vcpu now runs on the largest Boat
 * size when nothing fits, and two load tests wait for their sandbox.
 */
export const largestBoat = {
  label: nothingFitsLabel,
  boatSize: "xlarge",
  waiting: rows("largest-wait", [
    [
      "load test (api)",
      "Load",
      "mokly-cloud",
      "main",
      "waiting",
      none,
      "12s",
      nothingFitsLabel.name,
    ],
    [
      "load test (viewer)",
      "Load",
      "mokly-cloud",
      "main",
      "waiting",
      none,
      "12s",
      nothingFitsLabel.name,
    ],
  ]),
};
