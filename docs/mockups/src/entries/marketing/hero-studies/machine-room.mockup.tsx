import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { siteAddress } from "../../../fixtures/site.ts";
import { MachineRoom } from "../../../screens/marketing/hero-studies/machine-room.tsx";
import { LandingScreen } from "../../../screens/marketing/landing.tsx";

export const machineRoom = defineScreen({
  title: "Machine room",
  description:
    "Three machines at a release peak. The drawing shows the occupied slots and the jobs that moved to Boat.",
  rationale:
    "A cobalt technical drawing makes the hardware the main image. Each chassis stands for a machine in the release fixture.",
  relatedDocs: relatedMvp("architecture", "billing", "metering"),
  address: siteAddress(),
  render: (_props, context) => (
    <LandingScreen viewport={context.viewport} hero={<MachineRoom />} />
  ),
});
