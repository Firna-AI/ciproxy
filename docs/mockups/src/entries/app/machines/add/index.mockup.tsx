import { defineScreen } from "@mokly/mokly";

import { planDoc, stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { AddMachineScreen } from "../../../../screens/machines/add-machine.tsx";

export const addMachine = defineScreen({
  title: "Add a machine",
  description:
    "The same install command as in setup, for another machine. The page waits for the machine to connect.",
  relatedDocs: [planDoc],
  address: orgAddress("machines/new"),
  propSchema: stateSchema(["waiting", "connected"]),
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
  ],
});
