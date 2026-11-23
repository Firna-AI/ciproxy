import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../screens/settings/overflow.tsx";

export const overflow = defineScreen({
  title: "Overflow",
  description:
    "The settings that hold for every provider come first: Use overflow providers, the one switch of the workspace, and the wait for a free slot on the own machines, 30 seconds. A panel for each provider follows. Boat shows its account mokly, Change key, Disconnect, and its own monthly spend limit, with the costs that Boat confirmed and the costs that it has not confirmed yet apart. Second provider, the fixture name of a later provider, is available in the deployment but not connected, and Connect Second provider opens its form. The connection states are under Connection, the spend states under Spend, and two connected providers under Two providers.",
  rationale:
    "The user's decisions of 23 November 2026: a workspace can connect several providers, each with its own monthly spend limit, and the switch stays one switch for all providers. A cost that a provider has not confirmed counts at its bound until the provider reports it, so one combined figure would read as money already spent. A bound whose lifetime ended without a report stays with the unconfirmed costs, because the provider never confirmed it.",
  relatedDocs: related("overflow", "overflowSpend", "jobRouting", "labels"),
  address: orgAddress("settings/overflow"),
  useCasePaths: [
    paths.flowPeakOverflow,
    paths.flowSkipBoat,
    paths.flowChangeBoatKey,
  ],
  render: (_props, context) => (
    <OverflowScreen viewport={context.viewport} state="settings" />
  ),
});
