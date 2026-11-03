import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { planDoc, stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { OverflowScreen } from "../../../../screens/settings/overflow.tsx";

export const overflow = defineScreen({
  title: "Overflow",
  description:
    "The Boat account, the wait before a job starts on Boat (30 seconds), and the monthly Boat spend limit with the spend so far.",
  relatedDocs: [planDoc],
  address: orgAddress("settings/overflow"),
  useCasePaths: [paths.flowPeakOverflow],
  propSchema: stateSchema(["settings", "limit-reached", "boat-rejected"]),
  props: { state: "settings" },
  render: (props, context) => (
    <OverflowScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "limit-reached",
      title: "The spend limit is reached",
      description:
        "Boat spend reached the limit. Jobs wait for own machines until the month ends or the limit rises.",
      props: { state: "limit-reached" },
    },
    {
      slug: "boat-rejected",
      title: "Boat rejected the key",
      description:
        "Boat stopped accepting the API key. Jobs wait for own machines until a new key connects.",
      props: { state: "boat-rejected" },
    },
  ],
});
