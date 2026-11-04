import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

const address = orgAddress("machines");

export const empty = defineScreen({
  slug: "empty",
  title: "No machines yet",
  description:
    "An organization that skipped the machine step of setup. Add machine opens the install command.",
  relatedDocs: related("agent", "metering"),
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
  relatedDocs: related("agent", "metering"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="offline" />
  ),
});

export const overLimit = defineScreen({
  slug: "over-limit",
  title: "More own machines than the tier includes",
  description:
    "office-mini-1, a mini PC with a small disk, joined as the fourth machine on Team, which includes three. All four run jobs; the banner on every page offers the extra-machine add-on. office-mini-1 has 6 slots, but its disk holds 3 jobs of ciproxy at a time; its row opens its machine page, which says why.",
  relatedDocs: related("agent", "metering"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="over-limit" />
  ),
});
