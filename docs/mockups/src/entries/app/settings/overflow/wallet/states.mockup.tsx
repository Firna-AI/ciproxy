import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../../screens/settings/overflow.tsx";

const address = orgAddress("settings/overflow");
const docs = relatedMvp("overflow", "security");

export const connect = defineScreen({
  slug: "connect",
  title: "Choose the wallet of a first connection",
  description:
    "Boat accepted the key. The same wallet choice as in setup lists the personal wallet and each Boat organization wallet of the key, with the likely mokly wallet picked. Connect Boat connects the key, the wallet, and the limit.",
  rationale:
    "The owner's answer B to item 94, 8 November 2026. Setup and Settings under Overflow use one wallet choice.",
  relatedDocs: docs,
  address,
  useCasePaths: [paths.flowSkipBoat],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="wallet-connect" />
  ),
});

export const reconnect = defineScreen({
  slug: "reconnect",
  title: "Choose the wallet of a new key",
  description:
    "Boat rejected the saved key on 12 November, and the new key passed the check. The wallets of the new key show, with the likely one picked, and the limit keeps the saved $200. Disconnect stays beside Connect Boat.",
  relatedDocs: docs,
  address,
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="wallet-reconnect" />
  ),
});

export const changeKey = defineScreen({
  slug: "change-key",
  title: "Choose the wallet in Change key",
  description:
    "The new key passed the check. Its wallets show with the likely one picked, and the limit keeps the saved $200. Running Boat jobs finish with the current key.",
  relatedDocs: docs,
  address,
  useCasePaths: [paths.flowChangeBoatKey],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="wallet-change-key" />
  ),
});
