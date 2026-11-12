import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachineDetailScreen } from "../../../../screens/machines/machine-detail.tsx";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

const address = orgAddress("machines");

export const viewer = defineScreen({
  slug: "viewer",
  title: "Machines, seen by a Viewer",
  description:
    "Jonas Weber, a Viewer, sees the three machines, their slots, and the Boat spend, read-only. The page has no Add machine and no link to the overflow settings, and Settings is not in his navigation. A machine opens its page read-only.",
  rationale:
    "Only an Admin changes the machines and Boat, so the read-only roles see the state without the actions.",
  relatedDocs: related("security", "agent"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="steady" role="viewer" />
  ),
});

export const billing = defineScreen({
  slug: "billing",
  title: "Machines, seen by a Billing person",
  description:
    "Dana Kim has the Billing role. A fourth machine passed the three of Team, and the banner gives her its action, Plan and usage, because she pays. She cannot add a machine or open the overflow settings.",
  relatedDocs: related("security", "billing", "metering"),
  address,
  render: (_props, context) => (
    <MachinesScreen
      viewport={context.viewport}
      state="over-limit"
      role="billing"
    />
  ),
});

export const machineViewer = defineScreen({
  slug: "machine-viewer",
  title: "A machine, seen by a Viewer or a Billing person",
  description:
    "Jonas Weber, a Viewer, opens hetzner-ax42-1 from Machines. He sees its slots, warm runners, health, cache, and cost, read-only: the page has no Drain, Remove, Clear cache, or Edit, and the warm runner count has no buttons. A Billing person sees the same page.",
  rationale:
    "Only an Admin manages the machines, and Billing adds nothing on this page, so one screen serves both roles.",
  relatedDocs: related("security", "agent", "cache"),
  address: orgAddress("machines/hetzner-ax42-1"),
  render: (_props, context) => (
    <MachineDetailScreen
      viewport={context.viewport}
      state="healthy"
      role="viewer"
    />
  ),
});
