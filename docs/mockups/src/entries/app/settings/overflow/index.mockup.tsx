import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../screens/settings/overflow.tsx";

export const overflow = defineScreen({
  title: "Overflow",
  description:
    "The Boat account, the wait before a job starts on Boat (30 seconds), and the monthly Boat spend limit. The spend of the month shows the costs that Boat confirmed and the costs that it has not confirmed yet apart, each against the limit. The connection states are under Boat account, and the spend states under Spend.",
  rationale:
    "A cost that Boat has not confirmed counts at its bound until Boat reports it, so one combined figure would read as money already spent. A bound whose lifetime ended without a report stays with the unconfirmed costs, because Boat never confirmed it.",
  relatedDocs: related("overflow", "jobRouting"),
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
