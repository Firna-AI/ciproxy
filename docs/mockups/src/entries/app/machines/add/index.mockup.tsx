import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
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
  relatedDocs: related("agent", "security", "metering"),
  address: orgAddress("machines/new"),
  propSchema: stateSchema(addMachineStates),
  props: { state: "waiting" },
  render: (props, context) => (
    <AddMachineScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "connected",
      title: "A fourth machine connected",
      description:
        "hetzner-ax42-4 connected and runs jobs. Team includes three own machines, so the banner of every page offers the extra-machine add-on.",
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
    {
      slug: "no-runner-right",
      title: "An Admin without the runner right",
      description:
        "Ana Souza is an Admin, but GitHub does not let her manage the self-hosted runners of mokly-ai. In place of the command, she sees who can add a machine: an owner of the organization, or an Admin with the GitHub runner right. Check again reads her right on GitHub again.",
      rationale:
        "A machine gets the job secrets of the repositories, so adding one needs the Admin role and the runner right of the organization.",
      props: { state: "no-runner-right" },
    },
  ],
});
