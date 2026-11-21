import { defineScreen } from "@mokly/mokly";

import { related } from "../../components/related-docs.ts";
import { stateSchema } from "../../components/state-schema.ts";
import { siteAddress } from "../../fixtures/site.ts";
import {
  DocsHomeScreen,
  docsHomeStates,
} from "../../screens/docs/docs-home.tsx";

export const docsHome = defineScreen({
  title: "Docs home",
  description:
    "The first page of the docs at ciproxy.dev/docs: what ciproxy does, cards to the main pages, how a job runs, and what a team needs.",
  rationale:
    "The Starlight structure, themed with the Graphite tokens: the sidebar of sections, the top bar with search, the article, and On this page. The page in the sidebar takes a soft fill, as in the dashboard sidebar.",
  relatedDocs: related("architecture"),
  address: siteAddress("docs"),
  propSchema: stateSchema(docsHomeStates),
  props: { state: "page" },
  render: (props, context) => (
    <DocsHomeScreen viewport={context.viewport} state={props.state} />
  ),
  variants: [
    {
      slug: "search",
      title: "Search",
      description:
        "The search dialog over the docs home after a search for runs-on. Starlight builds the index with Pagefind; each result opens its heading.",
      props: { state: "search" },
    },
  ],
});
