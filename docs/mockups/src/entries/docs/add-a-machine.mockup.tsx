import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { AddMachineGuideScreen } from "../../screens/docs/add-machine-guide.tsx";

export const addMachineGuide = defineScreen({
  title: "Add a machine",
  description:
    "The guide that connects a Linux machine: what the machine needs, the install command and its token, the first job image, the machines that run fewer jobs, what follows the connection, and a repeat install.",
  rationale:
    "The answer E to item 124: the step that shows the connected machine says that a new machine first downloads the job image and takes jobs when it is ready. No text promises a time, because a slow link makes the download long. The steps use the names of the dashboard: Setup, Machines, and Add machine.",
  relatedDocs: relatedMvp("agentInstall", "agent", "jobRouting", "metering"),
  address: siteAddress("docs/guides/add-a-machine"),
  render: (_props, context) => (
    <AddMachineGuideScreen viewport={context.viewport} />
  ),
});
