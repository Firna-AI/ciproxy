# Terms acceptance and business checkout

Status: Active. On branch
`conductor/ciproxy-terms-acceptance-and-business-checkout`, no pull request
yet. The pull request that completes the work deletes this plan, as the
`plans` skill says.

The review of the legal pages in pull request #111 left five follow-up
items that need a decision. On 22 November 2026, the user answered them with
"these are all annoying questions - I don't care about it". Under that
delegation, the orchestrator agent chose one option for each item. Each
choice below is an agent call, not a user decision. This change applies the
five calls in one pull request and deletes the five items. It also records
in `plans/follow-ups/taxes-after-launch.md` that the tax decision of
4 November 2026 stays in force. The brief of 22 November 2026, sent on
behalf of the user, asks for this plan and its implementation in one draft
pull request into `main` that depends on #111.

The options of the five items come from the review of #111 and its finding
checks, so this plan runs no new options check. The change does not record
which version of the terms a person accepted, and it adds no tax, no
merchant of record, and no consumer withdrawal function.

## Agent calls

Each call is an agent call of 22 November 2026, under the delegation above.

- **Terms notice on every sign-in: option B.** The setup return and the
  GitHub sign-in links inside the dashboard pass the same terms notice and
  a Continue button before the GitHub sign-in starts. The pending setup
  stays in the sealed server binding. Rejected: A, an acceptance sentence
  limited to the sign-in page. C, a record of the accepted version at one
  common step. D, a sentence that a sign-in inside a running sign-in
  accepts nothing. E, a notice in the footer of every page, with acceptance
  by continued use. F, the accepted version in the sealed sign-in cookie.
  Reason: the terms keep acceptance at each sign-in, and only the paths that
  skipped the notice change.
- **Terms acceptance parties: option D.** The terms define "the customer",
  by its legal name, and "you", the person who signs in, one time, and name
  the right party in each clause. Each person accepts the terms for their
  own use. The first Admin also accepts them for the customer, and the
  customer lets each Admin accept later versions for it. The dialog that
  grants the Admin role names this power. Rejected: A, acceptance for the
  customer by any person with authority, with no proof of it. B, a separate
  agreement for each customer. C, separate records of the acceptance of
  each person and of the customer. E, a set date that binds the customer to
  each later version. F, the terms box of Stripe Checkout. Reason: the
  terms name who binds the customer and keep acceptance at each sign-in.
- **Privacy retention requirements: option B.** Each retention period of
  the privacy policy goes into "Data retention" of the security protocol,
  with a decision line. A comment above `backup_configuration` links to it.
  The security doc splits into linked pages. Rejected: A, the periods only
  in the privacy text. C, option B with the backup location in the
  Terraform code. D, reminders in a README and a comment. Reason: the
  periods get one reviewed home, as the other retention limits have.
- **Terms changes for unattended workspaces: option C.** A changed version
  of the terms applies 30 days after its publication, or at an earlier
  sign-in. Paid workspaces also get an email at their billing address in
  Stripe, which an operator sends. Free workspaces keep the next-sign-in
  rule. Rejected: A, the next-sign-in rule for every workspace. B, the
  30-day rule for every workspace, with no direct notice. D, new runners
  that wait until a person accepts the new version. Reason: paid
  workspaces have the most at stake and get a direct notice, with the same
  email as a price change.
- **Consumer purchase rules: option B.** Paid plans are for business use
  only. The terms and the checkout review say so. The consumer clause of
  the terms stays as a fallback. Rejected: A, the current checkout with
  each withdrawal handled by hand. C, an express request, a durable
  confirmation, and a withdrawal function. D, a merchant of record, such as
  Stripe Managed Payments. Reason: ciproxy serves GitHub organizations, and
  this option removes the consumer-only duties at Medium effort.
- **Taxes after launch: no change.** The decision of 4 November 2026 in
  [Billing](../docs/protocol/billing.md#tiers) stays in force: ciproxy
  collects no tax for now. The item stays open until a later sale needs a
  tax method.

## Design

Each design call below is my call of 22 November 2026.

- **The terms step.** My call: the terms step is a new static dashboard
  page at `/continue`. It shows the terms notice of the sign-in page from
  one shared part, and a "Continue with GitHub" button. Reason: the static
  export renders the page in full without its query, so nothing changes
  after it loads, and the sign-in page stays as it is.
- **The setup return.** My call: the setup return checks its input as now,
  then keeps the pending setup sealed in the one-use OAuth cookie for at
  most 10 minutes, and opens `/continue`. It starts no GitHub
  authorization. Reason: the pending setup stays in the sealed server
  binding, as option B says, and the cookie list of the privacy policy
  stays true.
- **Continue.** My call: the button of the terms step opens
  `/api/auth/github?resume=setup`. With a valid pending setup, that route
  starts the GitHub authorization with the setup purpose. Without one, or
  with an expired or unreadable one, it starts a plain sign-in. The plain
  route `/api/auth/github` of the sign-in page starts a plain sign-in, as
  now, and its new binding replaces a pending setup. Reason: only the
  Continue of the terms step resumes a setup, so a later plain sign-in in
  the same browser never takes over the setup of another person.
- **The links inside the dashboard.** My call: the API gives `/continue`
  as the reauthorization URL of the session and of each organization link.
  Reason: the API already owns the address of each fresh authorization, so
  the install page, the add-organization page, and Check again pass the
  terms step with no change to their own logic.
- **The grant dialog.** My call: a static line of the dialog names the
  terms power, with a link to the Terms. The trust notice of the API stays
  as it is. Reason: the line needs a link, and the API notice covers the
  runners only.
- **Who accepts a changed version.** My call: where the terms combine
  options C and D, the earlier or next sign-in that binds the customer is
  the sign-in of an Admin. Each person's own sign-in still binds that
  person. Reason: option D lets only an Admin accept a later version for
  the customer.
- **The email of a changed version.** My call: on the day of publication,
  an operator emails the billing address in Stripe of each workspace whose
  subscription is active or in the payment failed state. Reason: those
  workspaces have a paid tier, as
  [Subscription states](../docs/protocol/billing.md#subscription-states)
  says.
- **The privacy policy.** My call: the privacy policy says that Firna Ltd
  emails the billing address in Stripe about a price change or a new
  version of the terms. Reason: the policy names each use of personal data,
  and option C adds this use.
- **The business-use line.** My call: each checkout review with a pay
  button says that paid plans are for business use only, with a link to
  the Terms. This covers the move from Free and the move between paid
  tiers. Reason: both reviews start a paid purchase.
- **The split of the security doc.** My call: `docs/protocol/security.md`
  keeps sign-in, roles, App permissions, repository access, and key
  custody. `security-isolation.md` takes tenant and job isolation.
  `security-data.md` takes secrets, job data, and data retention.
  `terms-acceptance.md` takes the rules of the terms: who accepts, the terms
  notice, and new versions. Reason: each page stays under about 250 lines,
  the names follow the split pages of billing and job routing, and the
  terms rules form one topic. The first form of this call kept the terms in
  `security.md`, which then had 276 lines.
- **The backup location.** My call: the retention requirements state
  periods only. The backup location stays in the privacy policy and the
  default of Cloud SQL. Reason: option B covers the periods, and option C,
  which adds the location, was not chosen.
- **The effective date.** My call: both legal pages keep the effective date
  of the day that the pull request is ready, 22 November 2026 if it is
  ready today. The pull request description says that the date changes to
  the publication date if they differ. Reason: the brief asks for it, and
  nobody has signed in or paid yet.
- **An expired pending setup.** My call, from the mockups: when the pending
  setup is older than 10 minutes, Continue gives a plain sign-in, and the
  dashboard opens the page that the session asks for. After a first
  install, that is the install page, whose "Install on GitHub" recovers.
  Reason: the case needs more than 10 minutes on the terms step, and an
  existing page recovers it. Rejected: a longer life of the pending setup,
  and a new notice that the API would report.
- **The late approval.** My call, from the mockups: the flow "An owner
  approves later" keeps its step "at his next sign-in" with no terms step.
  Reason: that sign-in starts on the sign-in page or on a dashboard link,
  and both show the terms notice, so each start of it shows the notice.
- **The sign-in folder of the mockups.** My call, from the mockups: the
  folder `app/auth` gets an index with sign-in as its one canonical screen,
  and the states of sign-in, the install page, and the terms step move into
  child folders. Reason: a sixth screen on that page breaks the limit of
  five screens.
- **The GitHub mark.** My call, from the screens: the dashboard's
  "Continue with GitHub" button follows its "Sign in with GitHub" button,
  with no GitHub mark, so both pages look the same. Both buttons differ
  from their mockups, which show the mark, and the sign-in button has done
  so since it was added. Reason: the sign-in page is outside the five
  calls. `plans/follow-ups/sign-in-button-github-mark.md` asks the user.
- **Links out of a dialog or a review.** My call, from the screens: the
  Terms links of the grant dialog and of the checkout review open a new
  tab. The terms notice opens the terms in the same tab, as before. Reason:
  the dialog and the review keep their place and their work.
- **Repository choices in the privacy policy.** My call, from the docs:
  the privacy policy says that configuration, repository choices included,
  stays at most while the workspace exists. Reason: the repository choices
  of an organization go with its data 30 days after it removes the App,
  which can come before the workspace goes, and the requirement says "at
  most".
- **The headings of the terms.** My call, from the docs: "Your machines and
  Boat" and "Your data" keep their titles. Reason: a heading binds no party,
  each clause names the customer or "you", and the titles match the
  mockups.
- **Three long protocol docs.** My call, from the docs: `billing.md`,
  `workspaces.md`, and `agent.md` stay over about 250 lines in this branch,
  and `plans/follow-ups/protocol-docs-over-length-target.md` splits them
  later. Reason: they were over the target before this branch, and their
  splits move many links outside the five calls.

Contract owners:

- [Security](../docs/protocol/security.md), with its new linked pages.
- [Terms acceptance](../docs/protocol/terms-acceptance.md).
- [Billing](../docs/protocol/billing.md).
- [Deployment operations](../deploy/operations.md).
- The [site README](../ts/site/README.md), for the legal pages.
- The [dashboard README](../ts/app/README.md) and its guides.
- The [ciproxy-api README](../crates/ciproxy-api/README.md).

## Milestone 1: Mockups

Tags: mockup

Depends on: none

The sign-in area gets the terms step, and the flows pass it after each setup
return. The grant dialog and the checkout reviews get their new lines.

- [x] Add the terms step screen at `/continue`, with the terms notice of
      the sign-in screen from one shared part and a "Continue with GitHub"
      button.
- [x] Add the terms step to each user flow that passes a setup return or a
      GitHub sign-in link inside the dashboard. Point the "Sign in with
      GitHub" action of the install that ciproxy cannot confirm to it.
- [x] Add the line about later versions of the terms, with a link to the
      Terms, to the dialog that grants the Admin role. Update the texts of
      the entries and the flow that describe the dialog.
- [x] Add the business-use line, with a link to the Terms, to each checkout
      review with a pay button, in the known and the current-contract
      scenes.
- [x] Run `npm --prefix ts run mockups:check`. Smoke-test the changed pages
      at both widths.
- [x] Update the design and the TODOs of this plan for each problem that
      the mockups show.

Evidence: `.context/terms-acceptance-and-business-checkout/m1/`.

## Milestone 2: Documentation, protocol, and legal texts

Depends on: 1

The protocol docs state the requirements of the five calls. The legal texts
and the operator step follow them.

- [x] Split `docs/protocol/security.md` as the design says. Update each
      live link, comment, and mockup reference that names a moved section.
- [x] State each retention period of the privacy policy in "Data
      retention", with a decision line that says agent call. Add a comment
      above `backup_configuration` in
      `deploy/terraform/modules/foundation/data.tf` that links to it.
- [x] Add the terms requirements to "Sign-in and access": the notice before
      each sign-in, the terms step, who accepts, and when a changed version
      applies. Give each call a decision line that says agent call.
- [x] Add the requirement of the grant dialog to "Roles".
- [x] Add the business-use requirement to "Checkout" in
      `docs/protocol/billing.md`, with a decision line that says agent
      call.
- [x] Add the operator step for a new version of the terms to
      `deploy/operations.md`.
- [x] Change the terms: the definitions, the party of each clause, the
      acceptance, the changes, and the business use with the consumer
      fallback.
- [x] Add the use of the billing address in Stripe to the privacy policy.
- [x] Keep the effective date of both pages at the day that the pull
      request is ready.
- [x] Update the site README where a rule of the legal pages changes.
- [x] Run the site build, the site tests, and the Markdown check.

Evidence: `.context/terms-acceptance-and-business-checkout/m2/`.

## Milestone 3: Screens

Tags: ui

Depends on: 1

The dashboard gets the terms step page and the two new lines, as the
mockups show them. It needs only the copy of the mockups, so it runs beside
Milestone 2. Until Milestone 4, Continue on the terms step gives a plain
sign-in, because the API ignores the `resume` value.

- [x] Add the `/continue` page, with the terms notice of the sign-in page
      from one shared part and a button that opens
      `/api/auth/github?resume=setup`.
- [x] Add the terms line to the dialog that grants the Admin role.
- [x] Add the business-use line to the checkout review.
- [x] Update the dashboard README and the guides of these pages.
- [x] Run the typecheck, the lint, the export, and the Node tests.

Evidence: `.context/terms-acceptance-and-business-checkout/m3/`.

## Milestone 4: The setup return and the sign-in links pass the terms step

Depends on: 2, 3

The API sends each GitHub sign-in that does not start on the sign-in page
through the terms step.

- [x] Keep the pending setup of the setup return in the one-use OAuth
      cookie, and open `/continue`.
- [x] Start the setup purpose from a valid pending setup at
      `/api/auth/github?resume=setup`. Start a plain sign-in otherwise.
- [x] Give `/continue` as the reauthorization URL of the session and of
      each organization link.
- [x] Update the tests of the setup return. Add regression tests for the
      terms step, the resume, an expired or unreadable pending setup, a
      plain sign-in after a setup return, and the reauthorization URL.
- [x] Regenerate the API types with `npm --prefix ts run api:generate`.
- [x] Update the `ciproxy-api` README.
- [x] Run the targeted API tests, rustfmt, and clippy.

Evidence: `.context/terms-acceptance-and-business-checkout/milestone-4/`
and `.context/terms-acceptance-and-business-checkout/m4/`.

## Milestone 5: Dashboard fixtures and browser smokes

Tags: ui

Depends on: 4

The fixtures of the dashboard mirror the API again, and the browser smokes
pass the terms step.

- [ ] Give `/continue` as the reauthorization URL in the fixtures that
      mirror the API.
- [ ] Make the browser smokes that follow a GitHub sign-in link pass the
      terms step and its Continue.
- [ ] Update the sign-in rules in `ts/app/src/api/README.md` and the
      browser checks in `ts/app/scripts/README.md`.
- [ ] Run the typecheck, the lint, and the changed browser smokes.

## Milestone 6: Follow-up items

Depends on: 2, 3, 4

The five items close, and the other items stay true after the change. No
item names a fixture or a smoke of Milestone 5, so this milestone runs beside
it.

- [ ] Delete `plans/follow-ups/terms-notice-on-every-sign-in.md`,
      `plans/follow-ups/terms-acceptance-parties.md`,
      `plans/follow-ups/privacy-retention-requirements.md`,
      `plans/follow-ups/terms-changes-unattended-workspaces.md`, and
      `plans/follow-ups/consumer-purchase-rules.md`.
- [ ] Record in `plans/follow-ups/taxes-after-launch.md` that the decision
      of 4 November 2026 stays in force, and keep the item open.
- [ ] Update each other follow-up item that names a changed section, file,
      or decision.

## Milestone 7: Verification, commit, review, and pull request

Depends on: every other milestone

- [ ] Run the targeted tests and the complete gate named in `AGENTS.md`.
- [ ] Commit with Conventional Commits and push the branch.
- [ ] Review the complete local diff against `origin/docs/legal-pages-firna`,
      the branch of #111, with the `implementation-review` skill after the
      push. Report the findings, then apply the fix rule: fix the `fix now`
      findings, record follow-up items under `plans/follow-ups/` for the
      rest, re-review once, and report.
- [ ] Delete this plan when the `plans` skill says, under "Deleting the
      plan".
- [ ] Open the draft pull request into `main`. Its description starts with
      "Depends on #111" and gives one line for each item: the option, what
      changed, and the commit.
