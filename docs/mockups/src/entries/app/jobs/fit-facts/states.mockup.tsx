import { defineScreen } from "@mokly/mokly";

import { related } from "../../../../components/related-docs.ts";
import { orgAddress } from "../../../../fixtures/org.ts";
import { JobsLiveScreen } from "../../../../screens/jobs/jobs-live.tsx";

const address = orgAddress("jobs");

const rationale =
  "The API says that no place of the order fits a label, and which places the order holds, but not which resource is short. So the notice names the full request with fits, the places of the order, and a way forward. The known-data scene of Why jobs wait names the short resources of the story.";

export const nothingFitsRequest = defineScreen({
  slug: "nothing-fits-request",
  title: "No permitted place fits a label",
  description:
    "Two load tests of ciproxy-32vcpu wait. No machine of yours and no Boat size fits a job of 32 vCPU, 128 GB of memory, and 200 GB of free disk. The label still waits when nothing fits, so the notice also offers the largest size. Edit label opens the label for an Admin.",
  rationale,
  relatedDocs: related("labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="nothing-fits-request" />
  ),
});

export const machinesOnly = defineScreen({
  slug: "machines-only",
  title: "A label whose order holds no provider",
  description:
    "An Admin removed Boat from the order of ciproxy-32vcpu, so the order holds the own machines only. No machine of yours fits a job of its request, and the label runs on your machines only, so the notice offers no provider size. Edit label opens the label for an Admin.",
  rationale,
  relatedDocs: related("labels", "jobRouting"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="machines-only" />
  ),
});

export const largestHeld = defineScreen({
  slug: "largest-held",
  title: "A largest-size label while its provider is at its limit",
  description:
    "Thursday 26 November: Boat reached its spend limit. ciproxy-32vcpu runs on the largest size when nothing fits, and Boat is the first connected and available provider of its order whose largest size holds the image. Boat cannot take jobs now, so the notice gives the reason of Boat, its spend limit, and not that nothing fits. It also gives the date when Boat takes jobs again, 1 December, as the notice of the Boat limit does. Raise the limit opens the overflow settings for an Admin. The label list says Waits: Boat reached its spend limit for the label. The other labels run on the own machines, which have free slots.",
  rationale:
    "labels.md, A size that nothing can run: while a label with Run on the largest size waits because the provider that would run it cannot take jobs now, the label list and the Jobs notice give the reason of that provider, such as its spend limit. overflow-spend.md, At the limit: the Jobs page gives the date when the provider takes jobs again. The screen of a refusal of Boat gives the time when ciproxy tries Boat again in place of the date.",
  relatedDocs: related("labels", "jobRouting", "overflowSpend"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="largest-held" />
  ),
});

export const largestRefused = defineScreen({
  slug: "largest-refused",
  title: "A largest-size label while its provider refuses new sandboxes",
  description:
    "A what-if of Wednesday 25 November, 10:12: the start limit of the Boat plan is full, so Boat refuses new sandboxes until 10:13. ciproxy-32vcpu runs on the largest size when nothing fits, and Boat is the first connected and available provider of its order whose largest size holds the image. Boat cannot take jobs now, so the notice gives the reason of Boat and the time when ciproxy tries Boat again, 10:13. The notice has no action, because the refusal ends by itself. The label list says Waits: Boat refuses new sandboxes for the label. The other labels run on the own machines, which have free slots.",
  rationale:
    "overflow.md, Refused sandboxes: a provider that refuses new sandboxes takes no job until the retry time that it gives, and ciproxy tries it again while jobs wait. labels.md, A size that nothing can run: the Jobs notice gives the reason of the provider that would run the label. The notice of the label gives the retry time in the words of the notice of the Boat refusal, so both say the same.",
  relatedDocs: related("labels", "jobRouting", "overflow"),
  address,
  render: (_props, context) => (
    <JobsLiveScreen viewport={context.viewport} state="largest-refused" />
  ),
});
