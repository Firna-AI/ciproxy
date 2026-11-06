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

This live probe checks the facts needed by the open organization-binding
decision. It does not enable an add-organization route. Run it once as an
organization owner, then once as a member whose install needs approval.

The private credential file supplies `github_user_token`, `github_app_jwt`,
`github_app_slug`, `target_organization_id`, and `installation_proof_file`. Use
a GitHub App user token for the initiating person. Use a fresh App JWT. Keep the
proof file outside the repository. The probe writes it with mode 600. It
contains the generated state and a URL that only the operator opens. The URL
uses GitHub's documented `suggested_target_id` parameter on the
`/installations/new/permissions` path. GitHub still decides who can finish the
installation.

```sh
python3 crates/ciproxy-api/scripts/live_install_binding.py prepare
```

Open `install_url` from the private proof file. Record the actual setup return
as `observed_callback_url` in that file. Do not paste it into a log. For the
member run, have an owner approve the request after the initiating browser
closes. Capture the raw signed installation webhook in a private file. Add
`webhook_body_file`, `webhook_secret`, and `webhook_signature` to the private
credential file. The probe checks its signature before it checks the requester
and organization.

```sh
python3 crates/ciproxy-api/scripts/live_install_binding.py verify
```

The probe checks all request and installation pages. It does not trust a
callback installation ID without the initiating person's user-token proof.
Repeat with two simultaneous flows into different workspaces. Record whether
GitHub returns enough information to distinguish those flows. Record only
pass/fail facts under `.context`, without URLs or credentials.

GitHub documents the [installation request API][requests] and the
[suggested-target URL parameter][install-url]. The live check remains blocked
until an operator supplies the test App and organization credentials.

[requests]:
  https://docs.github.com/en/rest/apps/apps#list-installation-requests-for-the-authenticated-app
[install-url]:
  https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/migrating-oauth-apps-to-github-apps#prompt-users-to-install-your-github-app
