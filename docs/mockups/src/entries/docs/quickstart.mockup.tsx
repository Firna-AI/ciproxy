import { defineScreen } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { related } from "../../components/related-docs.ts";
import { siteAddress } from "../../fixtures/site.ts";
import { QuickstartScreen } from "../../screens/docs/quickstart.tsx";

export const quickstart = defineScreen({
  title: "Quickstart",
  description:
    "Five steps from sign-in to the first job on an own machine: install the GitHub App, run the install command of Add machine, connect Boat, change runs-on, and push.",
  rationale:
    "The code blocks are the same install command and workflow change that setup shows in the dashboard, so the docs and the product never disagree.",
  relatedDocs: related("agent", "labels", "jobRouting", "security"),
  address: siteAddress("docs/quickstart"),
  useCasePaths: [paths.flowLandingToSignIn],
  render: (_props, context) => <QuickstartScreen viewport={context.viewport} />,
});
