import type { LegalSection } from "./legal.ts";

export const privacySections: readonly LegalSection[] = [
  {
    id: "who",
    title: "Who we are",
    paragraphs: [
      "ciproxy is provided by Firna Ltd, a company registered in England and Wales with company number 09018453. Its registered office is 26 John Hogan V C Road, Manchester, M40 7QU, United Kingdom. This policy describes data handled by the public site, hosted service, and software on your machines. The hosted control plane schedules jobs. The dashboard shows and manages workspace data.",
      "Firna Ltd is the data controller for the personal data that the site, the dashboard, and the hosted service handle. If your organization needs a data processing agreement, email legal@ciproxy.dev. This policy covers the people who visit the site, email us, start or finish a sign-in to the dashboard, or have a role or an open request for access in a workspace.",
      // GitHub: its Data Protection Agreement, version October 2025, sections
      // 3.A and 3.G. Boat: its privacy policy, version 2026-10-12, PP-2 and PP-10.
      "The customer controls the content of its GitHub organizations, its Boat sandboxes, and its own machines. GitHub holds the organization content under the customer's agreement with GitHub, and Boat holds the sandbox content under the customer's agreement with Boat. The customer decides about requests for that data.",
    ],
  },
  {
    id: "stored",
    title: "What ciproxy stores",
    paragraphs: [
      "We keep a person record while that person has a workspace role or an open request for access. We check that person's own organization memberships and ownership through their GitHub sign-in. The record holds the GitHub user ID, login, display name, and avatar address, the IDs of the person's GitHub organizations with their owner status, and an encrypted GitHub user token for the hourly membership checks.",
      "We store workspace and organization records, workspace names, roles, access requests, repository choices, labels, and settings. We keep install tokens and durable records of actions that start a GitHub App installation. A label's settings include its VM size and allowed runner locations.",
      "Hosted job metadata includes the job name, workflow, repository, ref, result, location, times, label, and VM size that ran. It also includes the cache source and available cache counts. Cache counts are the number of restored entries and their total size.",
      "GitHub supplies run events, head repositories, default branches, and cache entry facts for job and cache decisions. Entry facts include the ID, ref, key, version, and creation time. These facts are separate from cache contents.",
      "Machine records include the host name, processor, threads, memory, disks, slots, and the jobs or warm runners in those slots. Reports include last contact, virtualization status, free disk, agent version, cache space, and cache hits. The machine page also shows repository Docker disk counts and space.",
      "We keep usage and savings records, including starts, periods, machine registrations, daily totals, and the monthly machine costs that you enter. Savings totals include job minutes and costs on machines and Boat. Usage covers every organization in the workspace.",
      "Boat records include the connected account and selected wallet, encrypted API key, spend limit, and key rejection date when applicable. Sandbox records track the sandboxes that ciproxy starts, their state, costs, pending cost bounds, and saved start fees. The saved fee can differ from a provider-confirmed charge.",
      "Billing records include plan and add-on choices, subscription and payment state, billing dates, and Checkout session references. Each paid plan-change quote stores its selection, amounts, current period, and proration time. We keep invoice check results and differences from the quote. Those error records contain no secret or card data.",
      "On your machines, the agent keeps VM images, machine-cache entries, and repository Docker disks. It saves cache rights and URL origins from the cache handoff. An origin is the scheme, host, and port. Full cache URLs and runtime tokens stay in host memory only.",
      "The site sets no cookies. Its docs keep the state of their sidebar in your browser's tab storage. The dashboard sets three cookies: one keeps you signed in for up to 30 days, one protects your requests for the same time, and one protects the GitHub sign-in step and keeps the install details that GitHub returns, for up to 10 minutes each time that it is set. Page scripts cannot read the sign-in cookie. The dashboard also keeps a few values in your browser: the end of your sign-in, the last workspace that you opened, and the progress of an install, a payment, or Setup.",
      "The site and the dashboard use no analytics service. Google Cloud, our host, logs each request to the site, the dashboard, the download site, and the hosted service. Each entry holds the IP address, the time, the browser, the full address with any values after the path, and the address of the page that sent the request. Such values can be a one-use GitHub sign-in code, a search in the dashboard, or a payment session reference. The control plane logs its work and its errors with the IDs of workspaces, machines, runners, and installations, but never job code, secrets, or logs. When you email us, we keep your message and address to answer you.",
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
      "We use GitHub sign-in, membership checks, workspace roles, and installation records to control access and connect organizations. We use repository choices to limit which jobs get a runner. We rely on the contract, and on our legitimate interest in keeping the service secure.",
      "The GitHub App asks for four permissions. Self-hosted runners, read and write, lets ciproxy start a runner for each job. Metadata, read, lets ciproxy list repositories. Members, read, lets ciproxy check your own organization memberships and ownership with your App user token. Actions, read, lets ciproxy read run and cache facts.",
      "Actions access lets the App read run logs and artifacts, but the control plane never reads them. The App asks for no other permission. Run and cache facts support safe cache writes, repository Docker disks, and cache restores.",
      "We use job metadata, labels, machine reports, and Boat records to place runners and show job state. Machine reports show capacity, health, cache use, and agent version. We rely on the contract.",
      "We use usage records and machine costs to count starts, show savings, and suggest a plan and add-ons. A suggestion does not change your plan or charge you. We rely on the contract, and on our legitimate interest in running and improving the service.",
      "We use billing records to manage subscriptions, show quotes, process confirmed plan changes, and compare invoices with quotes. An invoice difference produces an error event for the ciproxy operators. We email the billing address that the customer gave Stripe about a price change or a new version of the terms. We rely on the contract, and on our legal obligations, such as tax and accounting law.",
      "We also use data to secure the service, to prevent abuse, and to meet our legal duties. We rely on our legitimate interests and our legal obligations for these uses. We send no marketing, and we do not sell personal data or share it for advertising.",
      "Where this section names the contract, it means our contract with you when you are the customer. For other people, such as the staff of a customer or a visitor to the site, we rely instead on our legitimate interest in providing the service that the customer chose.",
    ],
  },
  {
    id: "processors",
    title: "Who processes data for ciproxy",
    paragraphs: [
      "GitHub provides sign-in, organization and repository facts, job delivery, logs, artifacts, and the GitHub cache. Boat runs overflow sandboxes in your own account and bills you. Stripe handles card payments, Checkout, subscriptions, invoices, and receipts.",
      "The ciproxy download site distributes the install script and agent releases. Google Cloud hosts the site, the dashboard, the download site, and the control plane with its database and logs. Cloudflare runs the DNS of ciproxy.dev, and Google Workspace hosts our email.",
      // Stripe: its Data Processing Agreement, updated 28 September 2026,
      // section 2. GitHub and Boat: the sources above "The customer controls".
      "Google Cloud, Cloudflare, and Google Workspace process data for us as our processors. Stripe is our processor when it runs Checkout, subscriptions, invoices, receipts, and the customer portal. Stripe also uses payment data for its own purposes, such as fraud prevention and legal checks, as a separate controller. GitHub and Boat are separate controllers for their own accounts and uses. For the content of the customer's organizations and sandboxes, they act for the customer, as Who we are says.",
      "The hosted service keeps its database in the United States, in the Google Cloud region us-central1, and its database backups in the Google Cloud multi-region us. Our providers, Google Cloud included, can also process data in other countries.",
      "When we or our processors move data out of the UK, we rely on the UK Extension to the EU-US Data Privacy Framework where the recipient is certified. Otherwise, we rely on the standard contractual clauses of the European Commission with the UK International Data Transfer Addendum. Transfers from the UK to the EEA rely on UK adequacy regulations. Where the EU GDPR applies, we rely on the EU-US Data Privacy Framework where the recipient is certified, and otherwise on the standard contractual clauses alone. GitHub and Boat move data under their own terms with the customer. For a copy of these safeguards, email legal@ciproxy.dev.",
      "Within a workspace, all roles can view job metadata for every organization and repository. This includes repositories that the person cannot read on GitHub. Admins control machines and Boat runners that handle jobs and their secrets for every organization. Billing persons can also manage plans, add-ons, and invoices.",
      "Only the ciproxy operators at Firna Ltd can reach the hosted systems. They use this access only to run, secure, and recover the service, and to answer requests under this policy. We disclose data to a public authority only when the law requires it, and we tell the customer first when the law allows. If Firna Ltd merges or sells its business, the data can pass to the new owner, and this policy keeps applying to it. We give data to no one else, except our professional advisers under a duty of confidentiality, or when you ask us to.",
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
      "GitHub App installation entries have no age limit. An entry that links to an organization is deleted with the data of that organization, or with the workspace data. An entry that never links to an organization stays.",
      "A sign-in record keeps the ID of the person, hashes of its tokens, and its end time. ciproxy deletes it when you sign out, or soon after the 30-day sign-in ends. Labels, settings, repository choices, and other configuration stay at most while the workspace exists, and go with the workspace data.",
      "Billing records stay at most while the workspace exists, and go with the workspace data. The payment record of each paid plan change keeps its quote for that time. Other expired quotes, and the references of finished Checkout sessions, can go sooner. Stripe keeps the invoices for Firna Ltd, and it keeps some payment records for its own legal duties.",
      "Operational logs, these request logs included, are kept for 30 days. We keep emails to us while they are useful, or until you ask us to delete them. We keep accounting records, such as invoices, for as long as tax and accounting law requires. The database keeps its 14 most recent daily backups, so a deleted record normally leaves the backups within 14 days. An operator can also take a manual backup before a risky change, and deletes it within 14 days. Apart from backups, these periods do not extend the limits in the table.",
    ],
  },
  {
    id: "rights",
    title: "Your rights",
    paragraphs: [
      "Send a request about your data to legal@ciproxy.dev. Describe the request, and give the details that help us find your data. For example, give your GitHub login and the workspace if you use the dashboard, the address that you emailed us from, or the date, time, and IP address of a visit. We may ask for information to confirm who you are. Under the UK GDPR, you can ask for a copy of your data, and ask us to correct it, delete it, or limit its use. For data that we use under the contract, you can also ask us to move it to another service. Some of these rights apply only in some cases. The law where you live can give you more rights.",
      "Where we rely on legitimate interests, you have the right to object to that use at any time, for reasons that concern your situation. We then stop that use, unless we have compelling legitimate grounds for it or need the data for a legal claim.",
      "We answer within the time that data protection law sets. Firna Ltd answers each request about its own data as the data controller. You can raise a concern with us at legal@ciproxy.dev, or complain to a data protection authority. In the UK, this is the Information Commissioner's Office, at ico.org.uk/make-a-complaint/ or 0303 123 1113.",
      "For data that the customer controls, as Who we are says, ask the customer. An Admin of the workspace can clear the cache of a machine. You can also send any request to legal@ciproxy.dev. We answer for our own data and for the data that Stripe processes for us, and we tell you who decides about the rest. Removing the GitHub App follows the retention rules in this policy.",
    ],
  },
  {
    id: "security",
    title: "Security",
    paragraphs: [
      "Each workspace has separate machines, labels, settings, keys, caches, and data. A machine runs jobs only for its own workspace. Each machine job uses a fresh VM, and each Boat job uses its own sandbox. Cache and Docker disk access stays separate by repository.",
      "A job VM cannot reach another job VM, the agent, its machine's host, a private network, or a cloud metadata service. It can reach its own machine cache and the internet.",
      "The Boat key is encrypted in the hosted service and is used only for its workspace's sandboxes. GitHub App and Stripe keys stay in the hosted control plane. Each machine credential stays on its machine, where only root can read it. Removing the machine revokes that credential at once.",
      "Connections to the hosted service use HTTPS. The database has no public address and accepts only encrypted connections. ciproxy stores GitHub user tokens encrypted, and keeps only hashes of sign-in tokens. Staff administer the hosted systems only through their own Google Cloud accounts. The database keeps its backups inside Google Cloud. If a breach puts personal data at risk, we tell the affected customers and the regulator as the law requires.",
    ],
  },
  {
    id: "changes",
    title: "Changes to this policy",
    paragraphs: [
      "We publish each update of this policy on this page, with a new effective date. An update applies from that date. When the law requires more notice, we give it. Before we use personal data for a new purpose that needs your consent, we ask for it.",
    ],
  },
  {
    id: "contact",
    title: "Contact",
    paragraphs: [
      "The service provider is Firna Ltd, company number 09018453, at 26 John Hogan V C Road, Manchester, M40 7QU, United Kingdom. Send privacy questions and requests to legal@ciproxy.dev. Report abuse or a security problem to abuse@ciproxy.dev. Firna Ltd has not appointed a data protection officer.",
    ],
  },
];
