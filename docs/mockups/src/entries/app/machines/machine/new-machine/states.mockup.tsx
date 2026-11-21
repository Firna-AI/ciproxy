import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../../components/paths.ts";
import { relatedMvp } from "../../../../../components/related-docs.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../../screens/machines/machine-detail.tsx";

export const preparing = defineScreen({
  slug: "preparing",
  title: "Preparing",
  description:
    "hetzner-ax42-4 joined minutes ago and gets its first job image. The notice says that jobs start on it when it is ready, and that jobs run on the other machines and on Boat until then. Slots shows no slot bar and no free slot, and the warm runner count is the saved count. Health shows the checks of the connected machine. The cache served no restore yet. Team includes three own machines, so the banner offers the extra-machine add-on.",
  rationale:
    "The answer E to item 124: a machine takes jobs only after it has its first job image, so the page says why it takes none and where the jobs run meanwhile. Preparing is work in progress, not a fault, so the dot is blue and the notice is information. No text promises a time, because a slow link makes the download long.",
  relatedDocs: relatedMvp("agent", "jobRouting", "metering"),
  address: orgAddress("machines/hetzner-ax42-4"),
  useCasePaths: [paths.flowNewMachineImage],
  render: (_props, context) => (
    <MachineDetailScreen viewport={context.viewport} state="preparing" />
  ),
});
