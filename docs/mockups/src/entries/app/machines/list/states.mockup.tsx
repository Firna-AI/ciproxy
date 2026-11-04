import { defineScreen } from "@mokly/mokly";

import { planDoc } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

const address = orgAddress("machines");

export const empty = defineScreen({
  slug: "empty",
  title: "No machines yet",
  description:
    "An organization that skipped the machine step of setup. Add machine opens the install command.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="empty" />
  ),
});

export const offline = defineScreen({
  slug: "offline",
  title: "A machine is offline",
  description:
    "ciproxy lost contact with hetzner-ax42-3. New jobs go to the other machines and to Boat.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="offline" />
  ),
});

export const overLimit = defineScreen({
  slug: "over-limit",
  title: "More own machines than the tier includes",
  description:
    "A fourth machine joined on Team, which includes three. All four run jobs; the banner on every page offers the extra-machine add-on.",
  relatedDocs: [planDoc],
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="over-limit" />
  ),
});
