import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { relatedMvp } from "../../../components/related-docs.ts";
import { stateSchema } from "../../../components/state-schema.ts";
import { host } from "../../../fixtures/org.ts";
import { InstallAppScreen } from "../../../screens/auth/install-app.tsx";

export const installApp = defineScreen({
  title: "Install the GitHub App",
  description:
    "After the first sign-in, before any organization uses ciproxy. The page lists the permissions of the app. Install on GitHub opens the GitHub page that picks the organization and its repositories, then returns to setup. The first install creates the workspace, which takes the name of the organization.",
  rationale:
    "GitHub owns the choice of organization and repositories, so ciproxy shows only what the app asks for and hands over to GitHub. The answer A to item 32 adds Actions, read, so that ciproxy reads the branch and the event of each run from GitHub.",
  relatedDocs: relatedMvp("workspaces", "security"),
  address: `${host}/install`,
  useCasePaths: [paths.flowFirstGreenJob],
  propSchema: stateSchema(["install", "pending-approval"]),
  props: { state: "install" },
  render: (props, context) => (
    <InstallAppScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "pending-approval",
      title: "Waiting for an owner to approve",
      description:
        "Lena Fischer, a member who is not an owner, asked to install the app. GitHub sends the request to the owners. When one approves, Lena becomes the first Admin, and Check again continues to setup, where she adds the machine herself.",
      rationale:
        "The answer to item 50: the Admin role alone adds a machine, so the first Admin needs no GitHub right to manage runners.",
      props: { state: "pending-approval" },
    },
  ],
});
