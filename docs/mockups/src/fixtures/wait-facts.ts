// Waiting jobs of mokly-ai for the wait reasons that come from current facts
// of the API: a label whose fit ciproxy cannot check yet, a label that starts
// on the largest size of Boat, the first provider of its order, and a full
// queue while Boat cannot take jobs.

import { none, rows, type Job } from "./jobs.ts";
import type { ProviderKey } from "./providers.ts";
import { labelNamed, nothingFitsLabel, type Label } from "./settings.ts";

/** Monday 23 November, 09:12: a machine report is missing. */
export const unknownFit = {
  label: labelNamed("ciproxy-large"),
  waiting: rows("unknown-fit-wait", [
    [
      "integration (postgres)",
      "CI",
      "mokly-cloud",
      "#331",
      "waiting",
      none,
      "52s",
      "ciproxy-large",
    ],
    [
      "integration (redis)",
      "CI",
      "mokly-cloud",
      "#331",
      "waiting",
      none,
      "52s",
      "ciproxy-large",
    ],
    [
      "bench (catalogue)",
      "Bench",
      "mokly",
      "main",
      "waiting",
      none,
      "31s",
      "ciproxy-large",
    ],
  ]),
};

/**
 * Friday 27 November, 16:02: ciproxy-32vcpu now runs on the largest size when
 * nothing fits. Boat is the first provider of its order whose largest size,
 * xlarge, holds the image, and two load tests wait for their sandbox.
 */
export const largestBoat: {
  label: Label;
  provider: ProviderKey;
  size: string;
  waiting: readonly Job[];
} = {
  label: { ...nothingFitsLabel, whenNothingFits: "largest-size" },
  provider: "boat",
  size: "xlarge",
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
