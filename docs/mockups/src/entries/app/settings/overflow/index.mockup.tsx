import { defineScreen } from "@mokly/mokly";

import { paths } from "../../../../components/paths.ts";
import { related } from "../../../../components/related-docs.ts";
import { stateSchema } from "../../../../components/state-schema.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import {
  OverflowScreen,
  overflowStates,
} from "../../../../screens/settings/overflow.tsx";

export const overflow = defineScreen({
  title: "Overflow",
  description:
    "The Boat account, the wait before a job starts on Boat (30 seconds), and the monthly Boat spend limit with the spend so far.",
  relatedDocs: related("overflow", "jobRouting"),
  address: orgAddress("settings/overflow"),
  useCasePaths: [paths.flowPeakOverflow],
  propSchema: stateSchema(overflowStates),
  props: { state: "settings" },
  render: (props, context) => (
    <OverflowScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "limit-reached",
      title: "The spend limit is reached",
      description:
        "Boat spend reached the limit. Jobs run on own machines only until the month ends or the limit rises.",
      props: { state: "limit-reached" },
    },
    {
      slug: "boat-rejected",
      title: "Boat rejected the key",
      description:
        "Boat stopped accepting the API key on 12 November. Jobs run on own machines only until a new key connects. The reconnect asks for the new key and the spend limit, and the limit starts with the saved $200.",
      props: { state: "boat-rejected" },
    },
    {
      slug: "not-connected",
      title: "Boat is not connected",
      description:
        "An organization that skipped the Boat step of setup. Connecting Boat here asks for the key and the monthly spend limit, as setup does, so a limit applies from the first job on Boat.",
      props: { state: "not-connected" },
    },
  ],
});
