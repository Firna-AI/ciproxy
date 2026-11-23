import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

export const loading = defineScreen({
  slug: "loading",
  title: "While machines load",
  description:
    "Skeleton rows keep the layout of the table until the first read returns.",
  relatedDocs: related("agent"),
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
  relatedDocs: related("agent"),
  address: orgAddress("machines"),
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="error" />
  ),
});

export const overflowRefresh = defineScreen({
  slug: "overflow-refresh",
  title: "When the overflow panel fails to refresh",
  description:
    "The list opened again, and the read of the Overflow panel failed after an earlier read in the same tab worked. The panel keeps the row of each provider, the spend, and the wait of that earlier read, and a warning above them offers Try again. The machines stay as they are.",
  rationale:
    "Only the panel failed, so its own warning names it, apart from the warning of a failed machines read. The panel reads again when the list opens and with Try again, not on the interval of the machines, because each read asks each provider for the costs of open starts.",
  relatedDocs: related("overflow", "overflowSpend", "agent"),
  address: orgAddress("machines"),
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="overflow-refresh" />
  ),
});
