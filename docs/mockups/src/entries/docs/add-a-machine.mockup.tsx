import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { AddMachineGuideScreen } from "../../screens/docs/add-machine-guide.tsx";

export const addMachineGuide = defineScreen({
  title: "Add a machine",
  description:
    "The guide that connects a Linux machine or a Mac: what each needs, the section Add a Mac, the install command and its token, the first job image, the machines that run fewer jobs, also a Mac with an older macOS, what follows the connection, and a repeat install. A Mac needs about 176 GB of free disk for the macOS image and the jobs.",
  rationale:
    "The answer E to item 124: the step that shows the connected machine says that a new machine first downloads the job image and takes jobs when it is ready. No text promises a time, because a slow link makes the download long. The steps use the names of the dashboard: Setup, Machines, and Add machine. The command shows a placeholder for the token, because a docs page cannot know the token of a workspace. The agent install doc says that the same command connects a Mac, and the job routing doc says that a Mac runs at most 2 job VMs at a time.",
  relatedDocs: relatedMvp("agentInstall", "agent", "jobRouting", "metering"),
  address: siteAddress("docs/guides/add-a-machine"),
  render: (_props, context) => (
    <AddMachineGuideScreen viewport={context.viewport} />
  ),
});
