# GitHub workers

This guide belongs to the
[ciproxy-control-plane README](README.md#github-workers).

Each reconciliation pass also starts a scale-set listener for each label of
an admitted organization with an active App installation. It stops the
listeners of
bindings that are gone. A deleted label keeps its listener while it has
unfinished jobs or a pending delivery. That listener acquires no new jobs.
An inactive organization gets no new runner or warm capacity. Its idle runners
use GitHub-first retirement. Received jobs keep their trusted preparation and
completion listeners through the final acknowledgement. Active sibling
organizations keep their runners and demand.

GitHub does not automatically retry failed webhooks.
A separate worker scans all pages of retained App delivery history each
minute. It groups retries by delivery GUID and uses the latest attempt.
It requests redelivery for each relevant event whose latest attempt did not
get the route's acknowledgement, HTTP 204. `/hooks/github` returns 204 only
after the App accepts the delivery.
GitHub counts every status from 200 to 399 as a success. A redirect or a
proxy page can return such a status before the route runs, so the worker
does not count it as delivered. An extra redelivery is safe, because the
handled events are idempotent. A lost event is not safe.
A proxy that rewrites the 204 makes the worker redeliver that event each
minute while GitHub keeps it in the history. Do not put such a proxy in
front of the route.
The App's durable lifecycle outbox handles failures after local persistence.
A separate startup and five-minute poll refreshes current installation
authority and repository snapshots. It needs no optional event subscriptions.

One supervised accounting worker does job timing repair, the metering
refresh, and the raw job purge, in that order.
The ten-second tick and listener job delivery only signal it.
Pending signals merge into one pass, and passes never overlap.
Readiness, runner operations, and workspace leases never wait for it.
An error or a panic in one workspace does not stop the next workspace.
The supervisor restarts the worker when it ends. Shutdown cancels it with
the other short-grace runtime work.

The runtime installs the mandatory gateway log guard.
`RUST_LOG` changes other verbosity without exposing serialized frames.

The [runner guide](runners.md#removal-and-take-out) gives the removal and
take-out policies.
The user selected spare-disk reservations for sticky Docker attachments.
Active reservations count against root admission until those jobs finish.
Live KVM and authenticated GitHub checks remain operator work.
