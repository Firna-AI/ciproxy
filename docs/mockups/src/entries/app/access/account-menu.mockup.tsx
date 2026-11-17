import { defineScreen } from "@mokly/mokly";

import { related } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../screens/jobs/jobs-live.tsx";

export const accountMenu = defineScreen({
  slug: "account-menu",
  title: "An owner claims the Admin role from the account menu",
  description:
    "Dana Kim has the Billing role and owns both organizations of the workspace, mokly-ai and mokly-labs, on GitHub. ciproxy offers her the Admin claim, so the avatar opens her account menu with her role and Claim the Admin role. After the claim she is an Admin, and Settings joins her navigation. An owner of only some organizations sees no claim in the menu while the workspace has an Admin.",
  rationale:
    "An owner claims the Admin role when the workspace has no Admin, or when the owner owns every organization of the workspace. So no workspace loses control of its runners, and no organization takes over another. ciproxy decides with the person's own GitHub sign-in, and the menu shows the claim only when ciproxy offers it, as the dashboard reads the claim of the session.",
  relatedDocs: related("security"),
  address: orgAddress("jobs"),
  render: (_props, context) => (
    <JobsLiveScreen
      viewport={context.viewport}
      state="steady"
      role="billing"
      accountMenu
    />
  ),
});
