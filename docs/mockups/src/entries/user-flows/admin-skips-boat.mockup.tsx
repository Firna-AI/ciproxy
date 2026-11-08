import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";

export const adminSkipsBoat = defineUseCase({
  title: "Skip Boat in setup, and connect it later",
  description:
    "Cal Moore has no Boat key at hand during setup. He skips the Boat step, runs the first job on his own machine, and connects Boat later in Settings under Overflow.",
  relatedDocs: related("overflow", "security"),
  steps: [
    {
      screenPath: paths.connectBoat,
      title: "The Boat step",
      description:
        "The machine connected, so setup asks for a Boat key and a monthly spend limit.",
    },
    {
      screenPath: paths.firstJobAfterSkip,
      title: "Skip for now",
      description:
        "The Boat step closes with a dash and a link to Settings under Overflow. The first job is next.",
    },
    {
      screenPath: paths.overflowNotConnected,
      title: "Connect Boat in Settings",
      description:
        "Settings under Overflow asks for the key and the monthly spend limit, as setup does.",
    },
    {
      screenPath: paths.overflow,
      title: "Boat is connected",
      description:
        "The page shows the account name and never the key, the wait, and the spend of the month against the limit.",
    },
  ],
});
