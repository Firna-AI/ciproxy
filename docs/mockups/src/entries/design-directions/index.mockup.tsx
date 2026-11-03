import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../components/state-schema.ts";
import { orgAddress } from "../../fixtures/org.ts";
import { directionRoutes } from "../../screens/directions/direction-screens.tsx";
import { MachinesScreen } from "../../screens/machines/machines-list.tsx";
import { DirectionScope } from "../../theme/direction-scope.tsx";
import { graphite } from "../../theme/directions/graphite.ts";

export const directions = defineScreen({
  title: "Design directions",
  description:
    "Three professional visual directions for the dashboard: A. Graphite, B. Midnight, and C. Enterprise. Each child page shows Machines, the live queue, and Savings in one direction, on mobile and desktop. This page shows Machines in Graphite.",
  rationale:
    "The directions share the product screens and change only the theme layer, so the chosen direction can apply to every screen with little work.",
  relatedDocs: [planDoc],
  address: orgAddress("machines"),
  render: (_props, context) => (
    <DirectionScope direction={graphite} routes={directionRoutes("graphite")}>
      <MachinesScreen viewport={context.viewport} state="steady" />
    </DirectionScope>
  ),
});
