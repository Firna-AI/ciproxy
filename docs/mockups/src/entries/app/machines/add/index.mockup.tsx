import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { relatedMvp } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import {
  AddMachineScreen,
  addMachineStates,
} from "../../../../screens/machines/add-machine.tsx";

export const addMachine = defineScreen({
  title: "Add a machine",
  description:
    "The same install command as in setup, for another machine. The page waits for the machine to connect. Revoke token ends the token before its hour ends.",
  rationale:
    "Design call 30: the command stays one line. The install script of get.ciproxy.dev installs the agent, then runs it with the arguments after the two dashes: install and the token. The answer C to item 93: the note under the command names the hosts that the installer supports, so a person learns them before preparing a machine.",
  relatedDocs: relatedMvp("agent", "security", "metering"),
  address: orgAddress("machines/new"),
  useCasePaths: [paths.flowNewMachineImage],
  propSchema: stateSchema(addMachineStates),
  props: { state: "waiting" },
  render: (props, context) => (
    <AddMachineScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "preparing",
      title: "A fourth machine gets its job image",
      description:
        "hetzner-ax42-4, a fourth AX42, connected and reported 16 threads, 64 GB, KVM ready, and 6 slots. It gets its first job image, so the line under its facts says that jobs start on it when it is ready. Open Machines shows it as Preparing. Team includes three own machines, so the banner offers the extra-machine add-on.",
      rationale:
        "The answer E to item 124: the page that shows a new machine says that it takes no job until it has its first job image. The machine did connect, so the check and the word connected stay. No text promises a time, because a slow link makes the download long.",
      relatedDocs: relatedMvp("agent", "jobRouting", "metering"),
      props: { state: "preparing" },
      useCasePaths: [paths.flowNewMachineImage],
    },
    {
      slug: "connected",
      title: "A fourth machine connected",
      description:
        "office-mini-1, a mini PC in the office, connected, has its job image, and runs jobs. Its disk holds 3 jobs of ciproxy at a time, fewer than its 6 slots, and the line under its name says so. Team includes three own machines, so the banner of every page offers the extra-machine add-on.",
      props: { state: "connected" },
    },
    {
      slug: "revoked",
      title: "After Revoke token",
      description:
        "Cal Moore revoked the token of the command. The command is gone from the page, and it adds no more machines. The machines that joined with it stay connected. Create a new command shows a command with a new token. Setup shows the same block after a revoke.",
      rationale:
        "A leaked command must not stay usable for the rest of its hour, so an Admin can end its token at once.",
      props: { state: "revoked" },
    },
  ],
});
