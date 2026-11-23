import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";

export const adminSkipsBoat = defineUseCase({
  title: "Skip the provider step in setup, and connect Boat later",
  description:
    "Cal Moore has no Boat key at hand during setup. He skips the provider step, runs the first job on his own machine, and connects Boat later in Settings under Overflow. Boat then goes last in the order of each Linux label.",
  relatedDocs: related("overflow", "overflowSpend", "security", "labels"),
  steps: [
    {
      screenPath: paths.connectProvider,
      title: "The provider step",
      description:
        "The machine connected, so setup asks for an overflow provider, with Boat picked, and for its key and a monthly spend limit on it.",
    },
    {
      screenPath: paths.firstJobAfterSkip,
      title: "Skip for now",
      description:
        "The provider step closes with a dash and a link to Settings under Overflow. The first job is next.",
    },
    {
      screenPath: paths.overflowNotConnected,
      title: "Connect Boat in Settings",
      description:
        "Settings under Overflow asks for a provider, then for the key and the monthly spend limit, as setup does.",
    },
    {
      screenPath: paths.overflowWalletConnect,
      title: "Choose the wallet",
      description:
        "Check key lists the wallets of the key, with the likely one picked. Connect Boat connects.",
    },
    {
      screenPath: paths.overflow,
      title: "Boat is connected",
      description:
        "The page shows the switch and the wait, then the Boat account, never the key, and the spend of the month against the limit on Boat. Boat went last in the order of each Linux label.",
    },
  ],
});
