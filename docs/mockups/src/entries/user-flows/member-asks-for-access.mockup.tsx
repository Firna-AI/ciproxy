import { defineUseCase } from "@mokly/mokly";

import { paths } from "../../components/paths.ts";
import { planDoc } from "../../components/state-schema.ts";

export const memberAsksForAccess = defineUseCase({
  title: "A member of the organization asks for access",
  description:
    "Lena Fischer is a member of mokly-ai on GitHub and signs in to ciproxy on 18 November. She has no role, so her sign-in asks the Admins for access, and Cal Moore gives her a role.",
  relatedDocs: [planDoc],
  steps: [
    {
      screenPath: paths.signIn,
      title: "Sign in with GitHub",
      description:
        "Lena signs in. She is a member of mokly-ai on GitHub, with no role in ciproxy yet.",
    },
    {
      screenPath: paths.noRole,
      title: "Waiting for an Admin",
      description:
        "Every page of mokly-ai shows only this one. Her sign-in sent the request to the Admins, whom the page names.",
    },
    {
      screenPath: paths.membersRequest,
      title: "An Admin gives a role",
      description:
        "Cal sees the request in Settings, under Members, picks Viewer, and gives access.",
    },
  ],
});
