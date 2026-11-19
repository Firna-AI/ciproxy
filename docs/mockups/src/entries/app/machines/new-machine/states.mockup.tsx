import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { relatedMvp } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

export const preparing = defineScreen({
  slug: "preparing",
  title: "A new machine gets its job image",
  description:
    "hetzner-ax42-4, a fourth AX42, joined minutes ago and gets its first job image. Its row says Preparing, and in place of a slot bar its slots say Getting the job image. Slots in use counts the 18 slots of the other three machines only, and Warm runners counts none on the new machine. Its row opens its machine page. Team includes three own machines, so the banner offers the extra-machine add-on.",
  rationale:
    "The answer E to item 124: one closed state of the API says whether a machine is offline, waits for its first image, or is ready, and routing reads the same state. A machine that waits for its image takes no job, so no figure counts its slots as free. No text promises a time, because a slow link makes the download long.",
  relatedDocs: relatedMvp("agent", "jobRouting", "metering"),
  address: orgAddress("machines"),
  useCasePaths: [paths.flowNewMachineImage],
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="new-machine" />
  ),
});
