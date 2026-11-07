# Live API verification

Run these checks only against the operator's test workspace. Keep the credential
file outside the repository. Use a private file mode, such as `chmod 600`. Do
not paste its contents into a log or a message.

The file is JSON. It supplies `base_url`, `workspace_id`, `session_cookie`, and
`csrf_token`. `base_url` is the dashboard origin without `/api`.
`session_cookie` is the value of `__Host-ciproxy`. The session endpoint returns
the CSRF token. The script prints only check names and statuses. It creates one
install token, then revokes that token. It does not add or remove a machine,
label, organization, or member.

```sh
export CIPROXY_API_CREDENTIALS_FILE=/private/path/dashboard-api.json
python3 crates/ciproxy-api/scripts/live_smoke.py
```

## Repeated GitHub authorization

Use the test App and a browser that already authorized it. No live GitHub
credential exists in this workspace, so these steps remain operator work.

1. Sign in through `/api/auth/github`.
2. Save that cookie as `previous_session_cookie` in the private file.
3. Install the App on a test organization, or wait for its owner to approve.
4. Follow the session response's `reauthorization_url` as top-level navigation.
   Record whether GitHub shows another consent page.
5. Save the returned current cookie and CSRF token in the private file.
6. Run the script again. It compares the first and current expiry times.
7. Repeat the authorization with two tabs. Close one callback tab before it
   receives its response. Check again from the remaining tab.
8. Confirm that the original 30-day expiry stays in effect.

Record the consent result, the build identity, and pass/fail status under
`.context`. Record no cookie, OAuth code, state, grant, or install command.
GitHub's current browser consent behavior still needs this live check.

## Mount and deployment

The executable belongs to the control-plane owner. It must mount the API at
`/api` on the same dashboard origin. It must supply the production GitHub
adapters, shared gateway, trusted image disk size, and maintenance worker as the
[crate guide](../README.md#quick-start) describes.

## GitHub installation binding

This live probe checks the approved state-only organization-binding flow.
Run it once as an organization owner. Then run it as a member whose install
needs approval.

The private credential file supplies `github_user_token`, `github_app_jwt`,
`github_app_slug`, `target_organization_id`, and `installation_proof_file`. Use
a GitHub App user token for the initiating person. Use a fresh App JWT. Keep the
proof file outside the repository. The probe writes it with mode 600. It
contains the generated state and a URL that only the operator opens. The URL
uses `/installations/new?state=...`. Pick the organization on GitHub.
The private target ID is an expected observation only. It is never sent in
that URL. GitHub still decides who can finish the installation.

```sh
python3 crates/ciproxy-api/scripts/live_install_binding.py prepare
```

Open `install_url` from the private proof file. Record the actual setup return
as `observed_setup_url` in that file. Do not paste it into a log. For the member
run, have an owner approve the request after the initiating browser closes.
Capture the raw signed installation webhook in a private file. Add
`webhook_body_file`, `webhook_secret`, and `webhook_signature` to the private
credential file. The probe checks its signature before it checks the requester
and organization.

```sh
python3 crates/ciproxy-api/scripts/live_install_binding.py verify
```

The probe checks all request and installation pages. It does not trust a
callback installation ID without the initiating person's user-token proof.
Repeat with two simultaneous flows into different workspaces. Verify that
each original person chooses the exact intended workspace.
Request or event evidence alone must leave the organization waiting. Record only
pass/fail facts under `.context`, without URLs or credentials.

GitHub documents the [installation request API][requests] and the
[installation entry flow][install-url]. The live check remains blocked
until an operator supplies the test App and organization credentials.

[requests]:
  https://docs.github.com/en/rest/apps/apps#list-installation-requests-for-the-authenticated-app
[install-url]:
  https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/migrating-oauth-apps-to-github-apps#prompt-users-to-install-your-github-app

## Jobs, Savings, and billing

The control plane mounts the page router and raw Stripe hook.
The [API guide](../README.md#jobs-savings-and-billing-composition) describes
the shared billing service and its Stripe invoice-preview adapter.

Use the same private credential path as the other API checks:

```sh
export CIPROXY_API_CREDENTIALS_FILE=/private/path/dashboard-api.json
python3 crates/ciproxy-api/scripts/live_pages.py read
python3 crates/ciproxy-api/scripts/live_pages.py review
```

For review, add `selection` with `tier` and `add_ons`. The add-ons hold
`start_blocks` and `machines`. Add `private_result_file` outside the repository.
The script writes payment and portal results there with mode 600. It prints
only route names and pass/fail status.
For a paid change, compare the monthly total and today's charge with Stripe's
invoice preview. Check the recurring day for an anchor on day 29 to 31 after
a shorter month. A downgrade can have a negative later credit and no charge
today. The preview must keep the existing paid-period dates.

Use a Stripe test-mode workspace for the payment actions. `checkout` opens one
payment. Open its URL from the private result file in a browser. Add the
returned
`checkout_session_id` to the private credential file. Run `return` until Stripe
confirms the payment. Run it while confirmation waits too. Confirm that Plans
offers no second checkout. Repeat with an unfinished checkout.

`change` sends `selection`. `addons` sends a separate `add_ons` value. `portal`
opens invoices and card management. `free` schedules the end of the paid plan.
Each command changes only this explicit test workspace. Confirm the proration,
payment-required state, original period dates, and portal cancel/undo behavior.

To replay an actual signed delivery, add `stripe_webhook_body_file` and
`stripe_webhook_signature_file` outside the repository. Run `webhook` before
Stripe's signature tolerance ends. The script sends the exact bytes to the
dashboard host's `/hooks/stripe`, without a browser cookie or CSRF token.
Register that exact URL in Stripe. Keep the signing secret in the host's secret
store. Record no secret, cookie, signature, provider body, or private URL in
verification evidence.

## Exact App registration

Use `PUBLIC_URL/api/auth/github/callback` for sign-in. Use
`PUBLIC_URL/api/github/setup` for the App setup return. PUBLIC_URL has no
trailing slash. Keep OAuth during installation OFF, expiring user tokens ON, and
redirect on update OFF. Select no extra repository subscriptions. M13 owns
`/hooks/github` and periodic repository refresh. Stripe uses `/hooks/stripe`.
There is no `/hooks` alias.

The setup server starts its own PKCE flow when fresh user proof is needed. The
browser finishes on `/`. Read the verified tagged `setup` outcome from
`/api/session`, without copying an installation ID into a frontend request. The
incoming App setup `state` and the later OAuth nonce are different. Capture the
original `/api/github/setup` navigation as `observed_setup_url` in the private
proof file. Never use the OAuth code callback as that input. A round-tripped
install state is an observation, not authorization.
