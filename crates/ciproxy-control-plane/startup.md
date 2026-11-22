# Startup and hosted services

This guide gives how the hosted service starts, the services that startup
builds, and the image facts of the process. The [crate README](README.md)
gives the commands, the routes, and the rest of the control plane.

## Startup order

Serving reads its settings from the environment and its secrets from files.
A missing setting stops the process with exit status 1 and names the setting.
The process then signs and fetches the pinned published manifest.
It reads `downloads.json` from the same release directory.
These files supply the exact root bytes, image ID, archive hash, and size.
A failed Linux read stops the process.
When the three macOS settings are set, the process then reads the macOS
release in the same way, from the private R2 store with presigned URLs.
When a macOS setting is missing, the key is malformed, or the read fails, it
logs this and starts with no macOS release.
The process then connects to PostgreSQL and applies the embedded migrations.
It binds `HTTP_BIND` only after these steps succeed.

## Hosted services

Startup builds the dashboard access service first. It owns sign-in,
workspace roles, and the App install authority. The machine gateway and the
GitHub App service use it. The dashboard API gets the same gateway, App
service, scale-set client, and App signer.
`MachineRuntime::new` requires an `InstallationApi`. Composition passes the
App service's App-JWT installation client, which take-out acceptance uses.
`MachineRuntime::new` also starts the runtime task supervisor, so call it
inside a Tokio runtime.
The API reads machine capacity from the last report of each machine: its
slots, its job disk minus active Docker leases, and the root bytes of its
selected image.
A report with unavailable Docker reservations keeps its slots and image,
but its dashboard disk capacity is unknown. The scaler keeps the machine's
label eligibility and starts no new runner there until reservations return.
A machine without a report has zero slots and unknown disk capacity.
API maintenance runs at startup and then every minute. A failed run logs a
warning and runs again.
The page routes share the same billing, metering, and overflow services.
Paid-plan review uses Stripe's invoice previews through the billing service.
It uses the current quoted Stripe period when stored renewal state is late.
Stripe returns the customer to the dashboard on `PUBLIC_URL`, with the
payer's workspace: `/checkout-return` with the exact Checkout session,
`/plans` after a cancel of Checkout, and `/billing` from the portal.
An independent worker retries durable last-organization cancellations and
pending paid-move payment work at startup and each minute. GitHub maintenance
failures do not delay that worker. Billing raises paid-move operator events
through `HostedBillingAlerts`: error-level tracing events with the
`ciproxy_billing::operator` target. They promise no email or page.
Billing and PostgreSQL calls run on blocking workers.

## Image facts

This one release is the Linux release of the process. The API's image disk
size, the scaler fallback, and Boat sizes come from it. Each machine's
selected local image still sizes the new roots of that machine.

The runtime holds at most one release for each platform. It offers each
machine the release of the machine's platform, and nothing when that
platform has no release. Each release has its own URL signer. The Linux
release uses the CDN key. The macOS release uses a presigned R2 URL that
expires after 2 hours. Without the R2 signer, the runtime does not offer
the macOS release, so no unsigned macOS URL reaches a Mac.
[Runtime setup](../../deploy/runtime.md#turn-on-the-runtime) says which
release file gives each field.
