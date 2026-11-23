import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { MachinesScreen } from "../../../../screens/machines/machines-list.tsx";

const address = orgAddress("machines");

export const twoProviders = defineScreen({
  slug: "two-providers",
  title: "Two connected providers",
  description:
    "A what-if: mokly-ai connected Second provider, the fixture name of a later provider, on 9 November. The Overflow panel has a row for each provider, with its account and its spend of November against its own limit: Boat against $200 and Second provider against $100. The overflow wait shows once, under the rows, because it holds for every provider.",
  rationale:
    "The user's decisions of 23 November 2026: a workspace can connect several providers, and each provider has its own monthly spend limit.",
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="two-providers" />
  ),
});

export const noProvider = defineScreen({
  slug: "no-provider",
  title: "No connected provider",
  description:
    "mokly-ai skipped the provider step of setup. The Overflow panel says that jobs run on the machines only, and an Admin gets Connect a provider, which opens the overflow settings. A Viewer and a Billing person see the same line without the button.",
  relatedDocs: related("overflow", "jobRouting"),
  address,
  render: (_props, context) => (
    <MachinesScreen viewport={context.viewport} state="no-provider" />
  ),
});
