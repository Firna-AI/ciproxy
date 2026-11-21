import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../../../components/related-docs.ts";
import { stateSchema } from "../../../../../components/state-schema.ts";
import { orgAddress } from "../../../../../fixtures/org.ts";
import { RepositoriesScreen } from "../../../../../screens/settings/repositories.tsx";

export const repositories = defineScreen({
  title: "Repository access of an organization",
  description:
    "Which private repositories of mokly-ai may use ciproxy runners. The Organizations tab opens it for each organization of the workspace. All 14 private repositories of mokly-ai can; GitHub decides which repositories ciproxy sees.",
  relatedDocs: relatedMvp("security"),
  address: orgAddress("settings/organizations/mokly-ai"),
  propSchema: stateSchema(["all", "selected"]),
  props: { state: "all" },
  render: (props, context) => (
    <RepositoriesScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "selected",
      title: "Selected repositories",
      description: "Only the checked repositories may use ciproxy runners.",
      props: { state: "selected" },
    },
  ],
});
