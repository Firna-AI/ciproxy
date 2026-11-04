import { defineScreen } from "@mokly/mokly";

import { related } from "../../../components/related-docs.ts";
import { orgAddress } from "../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../screens/jobs/jobs-live.tsx";

export const accountMenu = defineScreen({
  slug: "account-menu",
  title: "An owner claims the Admin role from the account menu",
  description:
    "Dana Kim has the Billing role and is an owner of mokly-ai on GitHub. The avatar opens her account menu, with her role and Claim the Admin role. After the claim she is an Admin, and Settings joins her navigation.",
  rationale:
    "An owner of the organization can always claim the Admin role, so an organization never loses control of ciproxy. ciproxy checks the ownership with the person's own GitHub sign-in.",
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
