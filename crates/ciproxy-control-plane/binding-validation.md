# Live binding validation

Use a disposable test workspace, private repository, and custom label.
The replacement probe deletes and recreates that label. It preserves its
name and settings, but its label and binding IDs change.

Use the private credential file and workflow fixture from the
[overflow smoke guide](overflow.md#live-smoke-checks). Add `binding_label`
with the custom label name. Add `binding_organization_id` with the ciproxy
UUID of the organization used for reactivation. The label must allow only
the destination of the probe. Keep database credentials in the existing
PostgreSQL environment or password file. Give the file mode `0600`.

Lifecycle commands also need `github_app_jwt` and `github_app_slug` in that
file. Use a fresh App JWT for each command. The commands use GitHub's
[documented App lifecycle endpoints](https://docs.github.com/en/rest/apps/apps).
They require the App owner's authority. The disposable workspace must keep
an active sibling organization throughout the removal scenario.

Restart commands need `kubectl`, its existing operator authentication, and
`kubernetes_namespace` in the private file. `kubernetes_context` is
optional. They run the existing
[control-plane rollout](../../deploy/operations.md#rotate-a-runtime-secret).
They capture command output without printing credentials or responses.

Run commands from the repository root:

```sh
export CIPROXY_OVERFLOW_CREDENTIALS_FILE=/PRIVATE/binding-smoke.json
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py --help
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py observe
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py replace own
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py replace boat
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py replace boat --restart
```

Set the label to own machines only before `replace own`. Set it to Boat
only before `replace boat`. Provide capacity for the requested label.
The probe starts a held job, replaces its label, and starts another job.
It checks both captured targets, retained old history, successful jobs,
and completed remote cleanup. For Boat, it also checks provider deletion.
Cleanup cancels unfinished fixture runs and restores overflow settings.
The replacement label keeps its new identity after the probe.

## Rollout

Apply `202611081000_registration_targets` before new runtime writes.
The store factory applies it at connection time. Replace all runtime
writers together. Restart their workers. Run `observe` before and after
the first reconciliation passes. Unknown idle runners can retire and need
replacement capacity. Unknown busy runners keep their jobs.

Run both replacement probes after deployment. `replace boat --restart`
restarts the control plane while a fixture job runs. It checks that the
runner keeps its captured target. Use `replace own --restart` for the
same own-machine check. `restart` can also run the rollout alone.
The local integration regressions inject remote-ID changes before and
after registration. They prove which ID was actually used. The live
probe does not claim that a runner survives deletion of its remote set.

## Resume and retained reinstall

Keep an active sibling organization in the test workspace. Save a baseline:

```sh
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py baseline \
  .context/scale-set-bindings/live-baseline.json
```

Run the suspended-label replacement scenario:

```sh
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py suspend
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py replace-inactive
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py resume
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py \
  verify-reactivation .context/scale-set-bindings/live-baseline.json
```

Save a new baseline before the retained reinstall scenario. Then run:

```sh
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py baseline \
  .context/scale-set-bindings/reinstall-baseline.json
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py remove
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py replace-inactive
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py reinstall \
  .context/scale-set-bindings/private-install.json --open-browser
python3 -B crates/ciproxy-control-plane/scripts/binding-smoke.py \
  verify-reactivation .context/scale-set-bindings/reinstall-baseline.json
```

Reinstall has one manual dependency. The organization owner must complete
GitHub's browser installation. The script writes its URL to the private
file. `--open-browser` opens it when a browser is available. Without that
flag, open the URL from the private file. The script waits up to 15 minutes
for a new active installation in the retained workspace. It uses no
unsupported REST installation route. Complete this scenario within the
30-day retention period.

Verification checks active execution admission, retained binding and
runner targets, and completed idle retirement. A listener can keep cleanup
pending until its final acknowledgement releases the session lease. Run
verification after that lease ends. Run `replace own` or `replace boat`
after reactivation to prove that the replacement label can run a job.
Keep the safe command output under `.context/`.

These steps need real GitHub, Boat, deployment, and database access.
Local fake services cannot prove the live result. The commands print no
credential or response body.
