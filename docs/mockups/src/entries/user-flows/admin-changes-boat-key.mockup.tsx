import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";

export const adminChangesBoatKey = defineUseCase({
  title: "Change the Boat key, then disconnect Boat",
  description:
    "Cal Moore replaces the Boat key of mokly-ai, and later disconnects Boat, its one provider. Each connection asks for the spend limit on Boat, and running Boat jobs always finish. The labels keep Boat in their order while it is not connected.",
  relatedDocs: related("overflow", "overflowSpend", "security", "labels"),
  steps: [
    {
      screenPath: paths.overflow,
      title: "The connected account",
      description:
        "Settings under Overflow has a panel for Boat, with the account mokly, Change key, Disconnect, and the spend limit on Boat.",
    },
    {
      screenPath: paths.overflowChangeKey,
      title: "Change key",
      description:
        "The Boat panel asks for the new key and the spend limit on Boat, which starts with the saved $200. Running Boat jobs finish with the current key.",
    },
    {
      screenPath: paths.overflowWalletChangeKey,
      title: "Choose the wallet",
      description:
        "Check key lists the wallets of the new key, with the likely one picked. Connect Boat replaces the key.",
    },
    {
      screenPath: paths.overflowDisconnect,
      title: "Disconnect asks first",
      description:
        "ciproxy starts no new jobs on Boat. The labels keep Boat in their order and skip it. The jobs that run on Boat finish, and ciproxy then deletes the key.",
    },
    {
      screenPath: paths.overflowNotConnected,
      title: "No provider is connected",
      description:
        "Jobs run on the own machines only. The page asks for a provider, then for a key and a limit again; the limit field starts with the saved limit.",
    },
  ],
});
