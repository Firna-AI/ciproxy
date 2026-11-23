# Terms acceptance

This page states who accepts the terms of service of ciproxy, the terms
notice before each sign-in, and when a new version of the terms applies.
[Sign-in and access](security.md#sign-in-and-access) states the other rules
of a sign-in.

## Who accepts

- The Terms are the terms of service of ciproxy on the
  [public site](../architecture.md#public-site).
- Each person accepts the current Terms for their own use at each sign-in.
- The first Admin of a workspace also accepts the Terms for the customer
  that holds the workspace. That Admin needs the authority to do so.
- The customer lets each Admin accept later versions of the Terms for it.
- **Decision:** the first Admin accepts the Terms for the customer, and
  each Admin can accept later versions for it. Rejected: acceptance for
  the customer by any person with authority, with no proof, a separate
  agreement for each customer, separate records of each acceptance, a set
  date that binds every customer to each later version, and the terms box
  of Stripe Checkout. Reason: the terms name who binds the customer and
  keep acceptance at each sign-in. Agent call of 22 November 2026, under
  the user's delegation.

## The terms notice

- Each page that starts a GitHub sign-in to the dashboard shows the terms
  notice first, with a link to the Terms and to the privacy policy.
- These pages are sign-in and the terms step at `/continue`.
- The terms step has a "Continue with GitHub" button. The GitHub sign-in
  starts only when the person selects it.
- GitHub opens the setup return after a person installs the ciproxy GitHub
  App, or asks the owners of an organization for the install. The setup
  return opens the terms step.
- The terms step keeps the pending setup of the return for 10 minutes.
  Within that time, Continue with GitHub continues the setup. After that
  time, Continue with GitHub gives a plain sign-in.
- A plain sign-in from the sign-in page drops a pending setup.
- Each link inside the dashboard that starts a new GitHub sign-in opens the
  terms step first. Such links are Check again on a page whose sign-in has
  no workspace, and Sign in with GitHub on the install page and on the
  page of Add an organization.
- **Decision:** every path that starts a dashboard sign-in shows the terms
  notice first, on sign-in or on the terms step with its Continue button.
  Rejected: an acceptance only on the sign-in page, a record of the
  accepted version, a notice in the footer of every page with acceptance by
  continued use, and the accepted version in the sign-in cookie. Reason:
  the terms keep acceptance at each sign-in, and only the paths that
  skipped the notice change. Agent call of 22 November 2026, under the
  user's delegation.

## New versions

- Each change of the Terms gives them a new version, which ciproxy
  publishes with a new effective date.
- A new version binds each person at their next sign-in.
- For a workspace on a paid [tier](billing.md#tiers), a new version binds
  the customer 30 days after its publication, or at an earlier sign-in of
  an Admin. Continued use after those 30 days accepts it.
- On the day of publication, an operator emails the billing address that
  the customer gave Stripe, as
  [Announce a new version of the terms](../../deploy/operations.md#announce-a-new-version-of-the-terms)
  says.
- For a workspace on Free, a new version binds the customer at the next
  sign-in of an Admin.
- Until a new version binds the customer, the version that the customer
  accepted last keeps applying, also while the jobs of the workspace run.
- **Decision:** a new version binds a paid workspace 30 days after its
  publication, with an email to its billing address, and a Free workspace
  at the next sign-in of an Admin. Rejected: the next-sign-in rule for
  every workspace, the 30-day rule for every workspace with no direct
  notice, and new runners that wait until a person accepts the new
  version. Reason: paid workspaces have the most at stake and get a direct
  notice, with the same email as a price change. Agent call of 22 November
  2026, under the user's delegation.

## Related docs

- [Security](security.md#sign-in-and-access): sign-in, the roles, and the
  first Admin of a workspace.
- [Billing](billing.md#checkout): the paid tiers, and the business use that
  each review with a pay button names.
- [Workspaces](workspaces.md#add-an-organization): how a person installs
  the ciproxy GitHub App, after which GitHub opens the setup return.
- [Architecture](../architecture.md#public-site): the public site that
  publishes the Terms.
- [Deployment operations](../../deploy/operations.md#announce-a-new-version-of-the-terms):
  the email of a new version of the Terms.
