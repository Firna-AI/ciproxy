import type { LegalSection } from "./legal.ts";

export const privacySections: readonly LegalSection[] = [
  {
    id: "who",
    title: "Who we are",
    paragraphs: [
      "ciproxy is provided by [legal entity name], at [registered address]. This policy describes data handled by the public site, hosted service, and software on your machines. The hosted control plane schedules jobs. The dashboard shows and manages workspace data.",
      "For each purpose, we act as [controller or processor roles for each purpose]. Customer processing follows [data processing agreement terms and link]. This policy covers [people and services covered by this policy].",
    ],
  },
  {
    id: "stored",
    title: "What ciproxy stores",
    paragraphs: [
      "We keep a person record while that person has a workspace role or an open request for access. We check that person's own organization memberships and ownership through their GitHub sign-in. The GitHub sign-in fields we store are [GitHub sign-in identifiers and attributes stored].",
      "We store workspace and organization records, workspace names, roles, access requests, repository choices, labels, and settings. We keep install tokens and durable records of actions that start a GitHub App installation. A label's settings include its VM size and allowed runner locations.",
      "Hosted job metadata includes the job name, workflow, repository, ref, result, location, times, label, and VM size that ran. It also includes the cache source and available cache counts. Cache counts are the number of restored entries and their total size.",
      "GitHub supplies run events, head repositories, default branches, and cache entry facts for job and cache decisions. Entry facts include the ID, ref, key, version, and creation time. These facts are separate from cache contents.",
      "Machine records include the host name, processor, threads, memory, disks, slots, and the jobs or warm runners in those slots. Reports include last contact, KVM status, free disk, agent version, cache space, and cache hits. The machine page also shows repository Docker disk counts and space.",
      "We keep usage and savings records, including starts, periods, machine registrations, daily totals, and the monthly machine costs that you enter. Savings totals include job minutes and costs on machines and Boat. Usage covers every organization in the workspace.",
      "Boat records include the connected account and selected wallet, encrypted API key, spend limit, and key rejection date when applicable. Sandbox records track the sandboxes that ciproxy starts, their state, costs, pending cost bounds, and saved start fees. The saved fee can differ from a provider-confirmed charge.",
      "Billing records include plan and add-on choices, subscription and payment state, billing dates, and Checkout session references. Each paid plan-change quote stores its selection, amounts, current period, and proration time. We keep invoice check results and differences from the quote. Those error records contain no secret or card data.",
      "On your machines, the agent keeps VM images, machine-cache entries, and repository Docker disks. It saves cache rights and URL origins from the cache handoff. An origin is the scheme, host, and port. Full cache URLs and runtime tokens stay in host memory only.",
      "The site and dashboard use [cookies and browser storage, if any]. Other collected data includes [analytics, operational logs, error reports, support messages, and related identifiers, if any].",
    ],
  },
  {
    id: "never",
    title: "What ciproxy never receives",
    paragraphs: [
      "The hosted control plane never receives, stores, or logs your job code, job secrets, job logs, or cache contents. GitHub sends code and secrets directly to the runner. Runners send logs and artifacts directly to GitHub. The control plane never reads those logs or artifacts.",
      "This hosted-service limit does not describe what jobs process on your machines or in Boat. Those jobs can handle code, secrets, and cache contents. The agent's machine cache and repository Docker disks keep local content between jobs.",
      "Stripe takes card details directly in Checkout and the customer portal. Card details never reach ciproxy.",
      "The hosted control plane receives your Boat API key and stores it encrypted. It never sends that key to a machine, job VM, or Boat sandbox. The dashboard never displays the saved key.",
    ],
  },
  {
    id: "use",
    title: "How ciproxy uses data",
    paragraphs: [
      "We use GitHub sign-in, membership checks, workspace roles, and installation records to control access and connect organizations. We use repository choices to limit which jobs get a runner. We rely on [lawful basis for sign-in, membership checks, access, and installation records].",
      "The GitHub App asks for four permissions. Self-hosted runners, read and write, lets ciproxy start a runner for each job. Metadata, read, lets ciproxy list repositories. Members, read, lets ciproxy check your own organization memberships and ownership with your App user token. Actions, read, lets ciproxy read run and cache facts.",
      "Actions access lets the App read run logs and artifacts, but the control plane never reads them. The App asks for no other permission. Run and cache facts support safe cache writes, repository Docker disks, and cache restores.",
      "We use job metadata, labels, machine reports, and Boat records to place runners and show job state. Machine reports show capacity, health, cache use, and agent version. We rely on [lawful basis for job scheduling, machine reports, caching, and Boat operations].",
      "We use usage records and machine costs to count starts, show savings, and suggest a plan and add-ons. A suggestion does not change your plan or charge you. We rely on [lawful basis for usage counting, savings reports, and plan suggestions].",
      "We use billing records to manage subscriptions, show quotes, process confirmed plan changes, and compare invoices with quotes. An invoice difference produces an error event for the ciproxy operators. We rely on [lawful basis for billing, payments, and invoice error records].",
      "Any other uses of data follow [other purposes, including any marketing and analytics]. Those uses rely on [lawful bases and consent choices for other purposes]. Our sale or sharing practices are [data sale or sharing practices].",
    ],
  },
  {
    id: "processors",
    title: "Who processes data for ciproxy",
    paragraphs: [
      "GitHub provides sign-in, organization and repository facts, job delivery, logs, artifacts, and the GitHub cache. Boat runs overflow sandboxes in your own account and bills you. Stripe handles card payments, Checkout, subscriptions, invoices, and receipts.",
      "The ciproxy download site distributes the install script and agent releases. We also use [hosting, database, monitoring, support, and other providers or recipients].",
      "Each recipient acts as [legal roles of GitHub, Boat, Stripe, and each other recipient]. Data is stored and processed in [storage and processing countries]. International transfers follow [transfer destinations, safeguards, and how to request details].",
      "Within a workspace, all roles can view job metadata for every organization and repository. This includes repositories that the person cannot read on GitHub. Admins control machines and Boat runners that handle jobs and their secrets for every organization. Billing persons can also manage plans, add-ons, and invoices.",
      "Access by ciproxy staff follows [staff access rules]. We disclose data for legal demands under [legal-demand disclosure policy]. Disclosure during a corporate change follows [corporate change disclosure policy]. Other disclosures are [other recipients and disclosure purposes, if any].",
    ],
  },
  {
    id: "retention",
    title: "Data retention",
    paragraphs: [
      "The table at the end of this section gives the decided limits for the listed records. It does not set a period for unlisted records. Finished-job metadata and daily savings totals expire after 90 days on every plan. A shorter visible history window does not shorten storage.",
      "Removing the GitHub App starts a 30-day deletion period for that organization's data. Removing the last organization also starts that period for workspace data, including labels and settings. Earlier normal expiry still applies. This period does not add 30 days to the normal 90-day limits.",
      "A reinstall within 30 days can restore data that has not expired. It does not restart or extend a retention period. After removal of the last organization, each machine needs installation again and Boat needs a new connection.",
      "Taking an organization out through Settings differs from removing the App on GitHub. An Admin take-out keeps its job history in the workspace under the normal limit. Suspending the App keeps the organization, data, machines, Boat key, and subscription.",
      "After a Boat key change, ciproxy keeps the old key only until the last sandbox that needs it ends. Disconnecting Boat or removing the last organization follows the same rule. This can keep the key briefly after the connection ends.",
      "Each job VM ends after its job. The agent never reuses that VM's disk. Machine caches and repository Docker disks follow separate local rules. Deleting or evicting a GitHub cache entry leaves the machine-cache copy available. Clear cache deletes one machine's cache entries, excluding Docker disks and GitHub entries.",
      "The machine cache frees space when a job VM needs it. The agent can delete unused Docker disks to free disk. The agent discards disposable Docker copies when their jobs end. Boat jobs get no persistent Docker disk.",
      "GitHub App installation entries have no age limit. Their retention treatment is [installation entry retention and relationship to organization or workspace removal].",
      "Sign-in session records follow [sign-in session retention period]. Configuration outside the listed removal rules follows [configuration retention period]. Billing records and quotes follow [billing and quote retention periods]. Operational logs, backups, support records, and accounting records follow [other record retention periods]. Backup deletion follows [backup deletion method and timing]. These periods do not extend the stated limits.",
    ],
  },
  {
    id: "rights",
    title: "Your rights",
    paragraphs: [
      "Send a request about your data to [privacy contact email]. We use [identity check method and request information] to handle a request. Your available rights are [applicable access, correction, deletion, portability, restriction, objection, consent withdrawal, and sale or sharing opt-out rights].",
      "We respond within [response deadline and extension rules]. Requests about data handled for a customer follow [controller and processor request-routing procedure]. Complaints follow [complaint process and competent privacy regulator, if applicable].",
      "Use the machine cache controls for local cache entries. Use the relevant provider's controls for data kept by GitHub, Boat, or Stripe. Removing the GitHub App follows the retention rules in this policy.",
    ],
  },
  {
    id: "security",
    title: "Security",
    paragraphs: [
      "Each workspace has separate machines, labels, settings, keys, caches, and data. A machine runs jobs only for its own workspace. Each machine job uses a fresh VM, and each Boat job uses its own sandbox. Cache and Docker disk access stays separate by repository.",
      "A job VM cannot reach another job VM, the agent, its machine's host, a private network, or a cloud metadata service. It can reach its own machine cache and the internet.",
      "The Boat key is encrypted in the hosted service and is used only for its workspace's sandboxes. GitHub App and Stripe keys stay in the hosted control plane. Each machine credential stays on its machine, where only root can read it. Removing the machine revokes that credential at once.",
      "We protect hosted data with [hosting security controls and encryption outside the Boat key]. Staff access uses [staff access controls]. Backups use [backup protection controls]. Incident notifications follow [incident notification procedure].",
    ],
  },
  {
    id: "changes",
    title: "Changes to this policy",
    paragraphs: [
      "We will notify you of updates through [privacy policy notice method]. Updates take effect after [privacy policy notice period]. Material changes follow [treatment of material privacy policy changes]. A new purpose follows [new-purpose consent requirements, if any].",
    ],
  },
  {
    id: "contact",
    title: "Contact",
    paragraphs: [
      "The service provider is [legal entity name], at [registered address]. Send privacy questions and requests to [privacy contact email]. Our privacy representative is [data protection officer or representative details, if applicable].",
    ],
  },
];
