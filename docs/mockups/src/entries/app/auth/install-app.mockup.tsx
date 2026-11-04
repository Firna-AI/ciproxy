import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../components/paths.ts";
import { related } from "../../../components/related-docs.ts";
import { stateSchema } from "../../../components/state-schema.ts";
import { host } from "../../../fixtures/org.ts";
import { InstallAppScreen } from "../../../screens/auth/install-app.tsx";

export const installApp = defineScreen({
  title: "Install the GitHub App",
  description:
    "After the first sign-in, before any organization uses ciproxy. Install on GitHub opens the GitHub page that picks the organization and its repositories, then returns to setup.",
  rationale:
    "GitHub owns the choice of organization and repositories, so ciproxy shows only what the app asks for and hands over to GitHub.",
  relatedDocs: related("security"),
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
        "Lena Fischer, a member who is not an owner, asked to install the app. GitHub sends the request to the owners. When one approves, Lena becomes the first Admin, and Check again continues to setup, where an owner adds the machine.",
      props: { state: "pending-approval" },
    },
  ],
});
