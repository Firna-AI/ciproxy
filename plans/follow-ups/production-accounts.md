# No production GitHub App or Stripe live mode

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the billing API and launch milestones of
  `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's request.
- **Location:** `deploy/runtime.md`, the "GitHub App" and "Secrets" rows.
  `crates/ciproxy-api/scripts/README.md`, "Exact App registration".
  `crates/ciproxy-billing/README.md`.
- **Reason:** Both accounts belong to the user, and production needs them
  before its runtime starts.

- **Terms:**
  - **PUBLIC_URL:** the dashboard origin of an environment, without a
    trailing slash. For production it is `https://app.ciproxy.dev`.
  - **Receipt email:** the email that Stripe sends to a customer after a
    successful payment.
- **Background:**
  - The main preview uses the test App `ciproxy-preview` and the Stripe
    sandbox "CI Proxy sandbox" in test mode.
  - The App asks for four permissions only: self-hosted runners read and
    write, metadata read, actions read, and members read.
  - The App registration needs these settings. The sign-in callback is
    `PUBLIC_URL/api/auth/github/callback`. The setup URL is
    `PUBLIC_URL/api/github/setup`. The webhook is `PUBLIC_URL/hooks/github`.
    OAuth during installation is off. Expiring user tokens are on.
    Redirect on update is off. No extra repository events are selected.
  - The billing service creates the four monthly USD prices and the
    restricted customer portal itself. Stripe sends events to
    `PUBLIC_URL/hooks/stripe`.
  - Receipt emails are off in the sandbox and have no API setting. Test
    payments never send them, so only a live payment shows the result.
- **Task:**
  1. The user creates the production App under the `Firna-AI`
     organization with the settings above. The App stays private until
     the site is published, as `site-and-app-publication.md` says.
  2. The user activates Stripe live mode. The user creates a live secret
     key and a webhook endpoint at `https://app.ciproxy.dev/hooks/stripe`.
  3. The user turns on receipt emails in the sandbox now and in the live
     account before the launch. Open
     <https://dashboard.stripe.com/settings/emails>. Then turn on
     **Successful payments** under **Payments**.
  4. Store the App ID, the client ID, and the slug as the GitHub variables
     `CIPROXY_APP_ID`, `CIPROXY_APP_CLIENT_ID`, and `CIPROXY_APP_SLUG` of
     the production environment.
  5. Store the App key, the App webhook secret, the OAuth secret, the
     Stripe key, and the Stripe webhook secret as versions of the secrets
     `ciproxy-production-<file>`, as `deploy/runtime.md` names them. Read
     each value from a private file. Never put a value in a log.
- **Impact if no action:** The production control plane cannot sign users
  in, start runners, or take payments.
- **Needs:** The owner access of the user to the `Firna-AI` organization
  and to the Stripe account. The production GitHub environment from
  `production-launch.md`.
- **Done when:** The production App exists with the settings above.
  Stripe live mode is active with its webhook. Receipt emails are on in
  the sandbox and in the live account. The production environment holds
  each variable and secret.
- **Related:** `production-launch.md` uses these values. `taxes-after-launch.md`
  is an open decision on tax collection with Stripe.
