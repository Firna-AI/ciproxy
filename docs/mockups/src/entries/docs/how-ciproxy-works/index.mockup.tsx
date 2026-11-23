import { defineScreen } from "@mokly/mokly";

import { relatedMvp } from "../../../components/related-docs.ts";
import { siteAddress } from "../../../fixtures/site.ts";
import { HowCiproxyWorksScreen } from "../../../screens/docs/how-ciproxy-works.tsx";

export const howCiproxyWorks = defineScreen({
  title: "How ciproxy works",
  description:
    "The docs page that explains ciproxy: the parts, a picture of the path of one job, a job on your machines, warm runners, slots and disk, overflow to Boat, the cache, and the job image. The picture shows GitHub, ciproxy, your machine, and the runner as the steps 01 to 04 of the list A job on your machines, and the other way of a job to your Boat account.",
  rationale:
    "Cal's decision of 23 November 2026: the picture is boxes in a CSS grid with an arrow between neighbours, from the tokens, so that the site builds it with plain HTML and CSS and no new dependency. It follows the job, from left to right on a desktop and from top to bottom on a phone, and its numbers are the steps of A job on your machines, so the list explains it. The figure takes its layout from its own width, because the sidebar and On this page take part of a wide window. The other way to Boat says when a job takes it: when your machines have no room and Boat can take the job. That stays true for a macOS job, which never goes to Boat, as the job routing doc says. The picture shows no setting values and no job image, and the text keeps them. Its text is in reading order in a figure with a caption, and search leaves it out. As Cal decided on the same day, the warm runners of a Mac serve ciproxy-macos. The answer C to item 78: a job can run its steps in any Docker image with the container: key.",
  relatedDocs: relatedMvp(
    "architecture",
    "jobRouting",
    "overflow",
    "agent",
    "cache",
  ),
  address: siteAddress("docs/how-ciproxy-works"),
  render: (_props, context) => (
    <HowCiproxyWorksScreen viewport={context.viewport} />
  ),
});
