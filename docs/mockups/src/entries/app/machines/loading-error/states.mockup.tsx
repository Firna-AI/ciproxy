import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

export const loading = defineScreen({
  slug: "loading",
  title: "While machines load",
  description:
    "Skeleton rows keep the layout of the table until the first read returns.",
  relatedDocs: [planDoc],
  address: orgAddress("machines"),
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="loading" />
  ),
});

export const error = defineScreen({
  slug: "error",
  title: "When machines fail to load",
  description:
    "The shared failed-read state with Try again. The machine detail uses the same state.",
  relatedDocs: [planDoc],
  address: orgAddress("machines"),
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="error" />
  ),
});
