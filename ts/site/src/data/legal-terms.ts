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
      "ciproxy is provided by Firna Ltd, a company registered in England and Wales with company number 09018453. Its registered office is 26 John Hogan V C Road, Manchester, M40 7QU, United Kingdom. The customer is the person or organization that holds a ciproxy workspace. These terms cover the site at ciproxy.dev, the dashboard at app.ciproxy.dev, the ciproxy GitHub App, the hosted control plane, and the downloads at get.ciproxy.dev. You accept their current version each time that you sign in with GitHub. You must have the authority to accept these terms for the customer. The control plane is the hosted service that schedules jobs.",
      "ciproxy runs GitHub Actions jobs on your own machines first. When those machines are busy, it can start runners for Linux jobs in your own Boat account. A runner takes one job. A workspace is the ciproxy account that holds your organizations, machines, settings, and plan.",
      "ciproxy serves GitHub organizations and private repositories only. Installing the GitHub App on a personal account creates no workspace. Public repositories do not get ciproxy runners. You choose which private repositories may use the service.",
      "The service depends on GitHub, Boat, Stripe, your machines, and network connections. Jobs can wait when no permitted runner can start. ciproxy does not rerun a job by itself.",
      `Features marked Coming later are not available yet. These features are ${laterFeatures}.`,
      "You must be at least 18 years old and have a GitHub account. Customers in any country can use the service, as long as their use breaks no sanction or export control.",
    ],
  },
  {
    id: "accounts",
    title: "Accounts, workspaces, and organizations",
    paragraphs: [
      "You sign in with GitHub. ciproxy has no password of its own. A sign-in lasts 30 days. During that time, ciproxy checks your organization memberships with GitHub each hour. A workspace role requires current membership of one of its organizations.",
      "A workspace has Admin, Billing, and Viewer roles. A Viewer can see jobs, machines, savings, and Plan and usage. Billing adds control of the plan, add-ons, and invoices. An Admin also controls machines, labels, settings, organizations, Boat, and roles.",
      "Every person with a workspace role sees job metadata for all repositories of all its organizations. This includes repositories that the person cannot read on GitHub. Give access only to people you trust with that visibility. You are responsible for the roles that your workspace gives, and for what each person with a role does in ciproxy.",
      "An Admin can add machines and connect Boat without a separate GitHub runner-right check. Those runners run jobs and their secrets for every organization in the workspace. The Admin role grants control over those runners. Use separate workspaces when organizations need separate runner control.",
      "Each organization belongs to at most one workspace. Its repository choices stay separate. An Admin can allow all private repositories or selected repositories that GitHub makes visible. Allowing all private repositories also includes new private repositories that ciproxy can see.",
      "Keep install tokens, machine credentials, and provider keys secure. An install token can add any number of machines to its workspace for one hour. An Admin can revoke it earlier. Connect only machines and accounts that you may use. If a credential leaks, revoke the install token, remove the machine, or replace the Boat key in Boat and in ciproxy. Report suspected unauthorized access to abuse@ciproxy.dev.",
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
      "Payments are not refunded, except where the law requires a refund. When a change lowers the monthly total, Stripe keeps the prorated difference as a credit for a later invoice. If the law where you live gives you a right to withdraw from a purchase, email legal@ciproxy.dev within the withdrawal period. Where the law allows, you then pay for the service that you used before you withdrew. If you think a charge is wrong, email legal@ciproxy.dev before you dispute it with your bank. We announce a price change in the dashboard at least 30 days before it applies to your plan, and you can cancel before then.",
    ],
  },
  {
    id: "machines",
    title: "Your machines and Boat",
    paragraphs: [
      "The agent runs on supported x64 Linux machines with KVM and on Apple Silicon Macs. Installation runs as root. The agent updates itself, and an update does not stop a running job. You are responsible for access to your machines and your Boat account, for their upkeep and security, and for their costs.",
      "Each job on a machine runs in a fresh VM. The VM ends after the job, and its disk is not reused. The agent keeps a separate machine cache. On Linux machines, paid plans also offer repository Docker disks that can keep Docker data between jobs.",
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
      "Do not use ciproxy to break a law, a sanction, an export control, or the rights of others, such as privacy or intellectual property. Do not use it for illegal content, such as child sexual abuse material or terrorist or violent extremist material. Do not use it for malware, botnets, fraud, spam, or phishing, or to attack, scan, or overload systems without permission. Do not mine cryptocurrency, and do not run jobs whose main purpose is to use compute rather than to build, test, or ship software. Do not resell or share access to ciproxy, and do not break the terms of GitHub, Boat, or Stripe.",
      "We can suspend an account or a workspace for misuse, as Ending the service says. Report abuse or a security problem to abuse@ciproxy.dev.",
      "Send copyright complaints and law enforcement requests to legal@ciproxy.dev. A copyright complaint names the work, the material that you say infringes it, where that material is, and how to reach you. We review each complaint, and we can remove the material. The person whose material we remove can ask for a review at the same address.",
    ],
  },
  {
    id: "data",
    title: "Your data",
    paragraphs: [
      "The privacy policy describes job metadata, machine reports, workspace records, billing records, and the decided retention limits. Job code and secrets go from GitHub directly to the runner. Runners send logs and artifacts directly to GitHub. The hosted control plane does not read those logs or artifacts.",
      "The hosted control plane does not receive, store, or log job code, job secrets, job logs, or cache contents. Software on your machines handles job content and keeps local caches. The hosted-service limit does not mean that your machines or Boat cannot receive job content.",
      "You keep the rights that you hold in your content. You give us permission to store, copy, process, and transmit it as needed to provide ciproxy, also through the providers that the privacy policy names. You must have the right to give this permission. If your organization needs a data processing agreement, email legal@ciproxy.dev.",
      "You are responsible for your content, your workflows, and what your jobs do. Keep your own backups of what you need, because the service can fail. Your code, job logs, and artifacts stay on GitHub, and the privacy policy says how to get a copy of the data that ciproxy holds. We can use aggregated data that identifies no customer or person to run and improve ciproxy.",
    ],
  },
  {
    id: "agent",
    title: "The open-source agent",
    paragraphs: [
      "The customer-side software is open source under Apache-2.0. It includes the agent, its connection protocol, the image definition, the install script, the cache sidecar, and the runner patch. Their licence governs their use.",
      "The hosted control plane, dashboard, and public site have no open licence. Public access to the repository does not give those parts an open licence. These terms give you the right to use the hosted parts through the service, for your workspace, and no other right in them. Third-party software keeps its own licence and terms.",
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
      "We can suspend or end the hosted service of an account or a workspace that breaks these terms, or whose use puts other people or the service at risk. There is no fixed notice period, and in urgent cases we can act without notice. To ask for a review, email legal@ciproxy.dev with your GitHub username, the workspace or organization name, and why the suspension should end.",
      "A suspension keeps the data of the workspace under the retention limits of the privacy policy. When we end the service of a workspace, we delete its data after 30 days, as after the removal of its last organization. A suspension or an end for a breach of these terms gives no refund. If we stop the service for another reason, we refund the unused part of the paid period.",
    ],
  },
  {
    id: "liability",
    title: "Warranties and liability",
    paragraphs: [
      "We provide the service as available. We make no uptime commitment and offer no service level.",
      "These terms do not remove any warranty, right, or liability that the law does not let us exclude.",
      "The laws of England and Wales govern these terms, and the courts of England and Wales can hear disputes. If you are a consumer, this does not remove any mandatory protection of the law where you live, or your right to bring a claim in your local courts.",
    ],
  },
  {
    id: "changes",
    title: "Changes to these terms",
    paragraphs: [
      "Each change gives these terms a new version, which we publish on this page with a new effective date. A changed version applies from your next sign-in. A sign-in lasts 30 days, so a change applies within 30 days while you keep using ciproxy. You accept the new version when you sign in with GitHub. A change does not alter the price of a billing period that has already started. If you do not accept a change, you can stop using ciproxy at any time, and a cancelled plan ends at the end of its paid period.",
    ],
  },
  {
    id: "contact",
    title: "Contact",
    paragraphs: [
      "The service provider is Firna Ltd, company number 09018453, VAT number GB191373695, at 26 John Hogan V C Road, Manchester, M40 7QU, United Kingdom. Send questions about these terms to legal@ciproxy.dev. Send formal notices by email to legal@ciproxy.dev, or by post to the registered office. Report abuse or a security problem to abuse@ciproxy.dev.",
    ],
  },
];
