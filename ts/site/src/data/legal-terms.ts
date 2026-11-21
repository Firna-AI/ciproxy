import { formatCount, formatMoney, listText } from "./format.ts";
import type { LegalSection } from "./legal.ts";
import { addOnPrices, features, tierOrder, tiers } from "./tiers.ts";

const paidPlans = tierOrder
  .filter((key) => tiers[key].price > 0)
  .map((key) => {
    const plan = tiers[key];
    return `${plan.name} costs ${formatMoney(plan.price)} per month. It includes ${formatCount(plan.starts)} starts per billing period and ${formatCount(plan.machines)} machines.`;
  });

const laterFeatures = listText(
  Object.values(features)
    .filter((feature) => !feature.launch)
    .map((feature) => feature.name),
);

export const termsSections: readonly LegalSection[] = [
  {
    id: "service",
    title: "The service",
    paragraphs: [
      "ciproxy is provided by [legal entity name], at [registered address]. The customer is [customer definition]. These terms cover [covered services]. You accept them by [terms acceptance method]. You must have [authority needed to act for the customer]. The control plane is the hosted service that schedules jobs.",
      "ciproxy runs GitHub Actions jobs on your own machines first. When those machines are busy, it can start runners in your own Boat account. A runner takes one job. A workspace is the ciproxy account that holds your organizations, machines, settings, and plan.",
      "ciproxy serves GitHub organizations and private repositories only. Installing the GitHub App on a personal account creates no workspace. Public repositories do not get ciproxy runners. You choose which private repositories may use the service.",
      "The service depends on GitHub, Boat, Stripe, your machines, and network connections. Jobs can wait when no permitted runner can start. ciproxy does not rerun a job by itself.",
      `Features marked Coming later are not available yet. These features are ${laterFeatures}.`,
      "You must meet [eligibility rules and minimum age]. The service is available to [supported customer types and locations].",
    ],
  },
  {
    id: "accounts",
    title: "Accounts, workspaces, and organizations",
    paragraphs: [
      "You sign in with GitHub. ciproxy has no password of its own. A sign-in lasts 30 days. During that time, ciproxy checks your organization memberships with GitHub each hour. A workspace role requires current membership of one of its organizations.",
      "A workspace has Admin, Billing, and Viewer roles. A Viewer can see jobs, machines, savings, and Plan and usage. Billing adds control of the plan, add-ons, and invoices. An Admin also controls machines, labels, settings, organizations, Boat, and roles.",
      "Every person with a workspace role sees job metadata for all repositories of all its organizations. This includes repositories that the person cannot read on GitHub. Give access only to people you trust with that visibility. Your role-grant duties are [customer duties for workspace access].",
      "An Admin can add machines and connect Boat without a separate GitHub runner-right check. Those runners run jobs and their secrets for every organization in the workspace. The Admin role grants control over those runners. Use separate workspaces when organizations need separate runner control.",
      "Each organization belongs to at most one workspace. Its repository choices stay separate. An Admin can allow all private repositories or selected repositories that GitHub makes visible. Allowing all private repositories also includes new private repositories that ciproxy can see.",
      "Keep install tokens, machine credentials, and provider keys secure. An install token can add any number of machines to its workspace for one hour. An Admin can revoke it earlier. Your credential duties are [customer credential duties]. Report suspected unauthorized access through [unauthorized-access reporting procedure].",
    ],
  },
  {
    id: "plans",
    title: "Plans and payment",
    paragraphs: [
      `Every workspace starts on Free. It includes ${formatCount(tiers.free.starts)} starts per monthly period and ${formatCount(tiers.free.machines)} machine. Free includes ${tiers.free.organizations} GitHub organization. It needs no card.`,
      ...paidPlans,
      `Paid plans can add ${formatCount(addOnPrices.startsBlock)} starts for ${formatMoney(addOnPrices.startsPrice)} per month per block. Each extra machine costs ${formatMoney(addOnPrices.machinePrice)} per month. Free cannot buy add-ons.`,
      "Prices are in US dollars. Prices exclude tax, and ciproxy collects no tax. Stripe bills paid plans monthly. Boat costs are separate and are billed by Boat to you.",
      "A start is a job that a runner takes on a machine or Boat, whatever its result. A rerun is another start. A runner that takes no job is not a start. Each registered machine counts, including offline machines and machines that cannot run jobs.",
      "Start and machine limits are soft limits. Exceeding them adds no charge and does not stop jobs. The organization limit on Free is also soft for existing active organizations. A new organization above that limit joins inactive and gets no runner until its reasons end.",
      "Only an Admin or a Billing person can choose a paid plan or change add-ons. ciproxy does not choose a paid plan, change quantities, or add an overage charge by itself. The dashboard shows the monthly total before you confirm a purchase.",
      "Stripe takes card details directly. A move from Free starts after Stripe confirms payment. Paid plan moves and saved add-on changes take effect at once. Stripe prorates the difference for the remaining period. A paid plan upgrade starts only after its charge succeeds.",
      "A paid plan move has a stored quote. Its review shows the new monthly total, today's prorated amount, and later billing dates. Each new review needs your confirmation. The quote keeps the amounts and proration time for a retry of that move.",
      "You can cancel in the Stripe customer portal or choose Move to Free. The paid plan and add-ons remain until the paid period ends. You can undo cancellation before then. While Stripe retries a failed payment, the paid plan remains active.",
      "After the final failed retry, the subscription ends and the workspace moves to Free. Jobs keep running on all its machines, including above Free's soft limits. A plan change controls the history window shown in the dashboard. The privacy policy states the storage limits.",
      "Refunds and credits follow [refund and credit policy]. Any withdrawal rights follow [applicable withdrawal rights and procedure]. Send disputed charges through [charge dispute procedure]. Future price changes follow [price change notice and timing].",
    ],
  },
  {
    id: "machines",
    title: "Your machines and Boat",
    paragraphs: [
      "The agent runs on supported x64 Linux machines with KVM. Installation runs as root. The agent updates itself, and an update does not stop a running job. You are responsible for [machine access, maintenance, provider account, and cost duties].",
      "Each job on a machine runs in a fresh VM. The VM ends after the job, and its disk is not reused. The agent keeps a separate machine cache. Paid plans also offer repository Docker disks that can keep Docker data between jobs.",
      "A machine cache can keep an entry after GitHub deletes or evicts it. Clear cache removes the entries of one machine. It does not delete GitHub cache entries or the machine's Docker disks. A job on Boat uses the GitHub cache and gets no persistent Docker disk.",
      "You connect your own Boat account with its API key and choose its wallet. The wallet serves every organization in the workspace. ciproxy stores the key encrypted and uses it only for your workspace's sandboxes. The dashboard does not show the key after saving it.",
      "You set a monthly Boat spend limit. It covers sandboxes that ciproxy starts, not other use of your Boat account. The limit counts confirmed costs and pending bounds. At the limit, new Boat runners wait, while current jobs finish. Their costs can take spend above the limit.",
      "Each Boat sandbox has a lifetime of six hours, or two hours on a Boat free trial. Boat ends it at that time. A job can end sooner. Boat bills your account for its sandboxes, separately from the ciproxy plan.",
      "Changing the Boat key or selecting Disconnect stops use of the old key for new runners. Current Boat jobs finish. ciproxy keeps the old key only until the last sandbox that needs it ends.",
    ],
  },
  {
    id: "use",
    title: "Acceptable use",
    paragraphs: [
      "Use ciproxy only with organizations, repositories, machines, accounts, and content that you have authority to use.",
      "Use the service lawfully. Respect other people's rights. Do not access another customer's workspace or data without authority. Do not interfere with the service or bypass its access controls.",
      "Additional prohibited uses are [additional use restrictions, if any]. We handle suspected misuse under [investigation, notice, enforcement, and appeal procedure].",
      "Send copyright complaints to [copyright complaint contact]. Include [required complaint information]. We handle complaints under [copyright review procedure]. A counter-notice follows [counter-notice procedure, if applicable].",
    ],
  },
  {
    id: "data",
    title: "Your data",
    paragraphs: [
      "The privacy policy describes job metadata, machine reports, workspace records, billing records, and the decided retention limits. Job code and secrets go from GitHub directly to the runner. Runners send logs and artifacts directly to GitHub. The hosted control plane does not read those logs or artifacts.",
      "The hosted control plane does not receive, store, or log job code, job secrets, job logs, or cache contents. Software on your machines handles job content and keeps local caches. The hosted-service limit does not mean that your machines or Boat cannot receive job content.",
      "You keep the rights that you hold in your content. You give us permission to process it to provide ciproxy. That permission has [processing permission scope, duration, and recipients]. Customer processing follows [data processing agreement terms and link].",
      "You are responsible for [customer content duties]. Backups and exports follow [backup and export terms]. Any use of aggregated or de-identified data follows [aggregate and de-identified data terms, if any].",
    ],
  },
  {
    id: "agent",
    title: "The open-source agent",
    paragraphs: [
      "The customer-side software is open source under Apache-2.0. It includes the agent, its connection protocol, the image definition, the install script, the cache sidecar, and the runner patch. Their licence governs their use.",
      "The hosted control plane, dashboard, and public site have no open licence. Public access to the repository does not give those parts an open licence. Use of hosted parts follows [hosted-service licence terms]. Other software follows [separate third-party licence terms].",
    ],
  },
  {
    id: "ending",
    title: "Ending the service",
    paragraphs: [
      "Drain makes a machine finish its jobs and take no new ones. Remove revokes its credential at once and stops warm runners. The guest is the ciproxy program that supervises the runner. A job taken just before the stop can fail before the guest receives it. Jobs already received by the guest finish.",
      "An Admin can take an organization out through Settings, except the last organization. This removes the App through GitHub and stops new runners for that organization. Running jobs finish. Its job history stays in the workspace under its normal retention limit.",
      "Removing the App on GitHub stops new jobs for that organization. Warm runners stop at once, with the same risk for a just-taken job. Other running jobs finish. Started Boat runners can remain, even when idle, until their exit report or sandbox lifetime ends.",
      "An idle Boat sandbox can therefore keep billing after App removal. Its lifetime remains six hours, or two hours on a Boat free trial. ciproxy deletes a sandbox whose runner has not started.",
      "When the last organization removes the App, ciproxy revokes all workspace machine credentials and install tokens at once. It starts no new Boat runner. Current Boat jobs finish. The Boat key is deleted when the last sandbox ends.",
      "The last organization's App removal ends the subscription at the end of the paid period. Another organization's removal leaves it unchanged. Organization data is deleted after 30 days. Removal of the last organization also deletes workspace data after 30 days. Earlier normal expiry still applies.",
      "A reinstall within 30 days can recover data that has not expired. It does not extend the normal retention periods. After removal of the last organization, machines need installation again and Boat needs a new connection.",
      "Suspending the App stops new jobs for that organization while current jobs finish. Its workspace membership, machines, Boat key, subscription, and data remain. Jobs can start again after the App is unsuspended.",
      "We can suspend or end hosted service for [provider suspension or termination grounds]. We give [suspension or termination notice period and method]. You can appeal through [suspension or termination appeal procedure]. Access to data follows [data access after suspension or termination]. Prepaid fees follow [prepaid fee treatment after suspension or termination].",
    ],
  },
  {
    id: "liability",
    title: "Warranties and liability",
    paragraphs: [
      "We provide the service with [service warranties and service-level commitments]. We exclude [warranty exclusions, if any], subject to [mandatory warranty rights and exceptions].",
      "Our liability for a claim is limited to [liability cap and calculation period, if any]. We are not liable for [excluded losses, if any]. These limits do not apply to [liability exceptions and mandatory rights]. Any indemnity follows [indemnity obligations, procedures, and limits, if any].",
      "The governing law is [governing law]. Disputes follow [courts or dispute procedure]. These terms preserve [mandatory customer rights].",
    ],
  },
  {
    id: "changes",
    title: "Changes to these terms",
    paragraphs: [
      "We will notify you of changes through [terms notice method]. Changes take effect after [terms notice period]. Your acceptance follows [acceptance rule for changed terms]. Existing subscriptions follow [existing subscription treatment after changes]. You may end service after a change under [right to end service after a change].",
    ],
  },
  {
    id: "contact",
    title: "Contact",
    paragraphs: [
      "The service provider is [legal entity name], at [registered address]. Send questions about these terms to [legal contact email]. Send formal notices through [formal notice address and method].",
    ],
  },
];
