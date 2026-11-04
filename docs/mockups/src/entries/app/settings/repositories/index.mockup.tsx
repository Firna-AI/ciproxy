import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { RepositoriesScreen } from "../../../../screens/settings/repositories.tsx";

export const repositories = defineScreen({
  title: "Repository access",
  description:
    "Which private repositories may use ciproxy runners. All 14 private repositories of mokly-ai can; GitHub decides which repositories ciproxy sees.",
  relatedDocs: related("security"),
  address: orgAddress("settings/repositories"),
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
