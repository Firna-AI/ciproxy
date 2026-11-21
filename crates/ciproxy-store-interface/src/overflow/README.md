# Overflow store

These traits and types form the overflow area of
[ciproxy-store-interface](../../README.md).

OverflowStore adds exclusive workspace sessions for Boat execution.
Pending connections use fresh store-issued grants before external work.
Last-App removal revokes these grants, and activation consumes one live grant.
Reusing a key-version identity cannot reuse an earlier grant.
Pending acceptance retains immutable demand, sealed input, and its saved fee.
It fixes no provider parameters and counts no provider cost bound.
Prepared dispatch carries the selected provider parameters and exact monthly
amounts together.
Its transaction locks saved settings and returns a distinct spend-limit refusal.
Its startup query derives the organization and key from the saved delivery.
It checks tenant ownership, active App state, enabled overflow, and the
accepted non-retired key together.
It also requires the saved scale set and the label's current Boat switch.
All current label dimensions must match the original requested dimensions.
Each delivery retains its image root bytes and required total root bytes.
Authority compares catalog disk with those saved totals.
Even a largest-size fallback must hold the complete image.
Reservation uses the same policy as saved-delivery startup.
The current-label query supplies resource policy facts without store policy.
The atomic dispatch operation rechecks authority and budget before `Creating`.
It preserves all pending fields and fixes size, lifetime, and dispatch time.
Later recovery uses those exact committed parameters.
Unsent deliveries can retry preparation without provider recovery.
Cancelled unsent deliveries retain no sealed input or callback authority.
Account observations keep typed catalogs, retry times, and refusal state.
Runner input stays encrypted until launch or cleanup ends its use.
Callback lookup derives the workspace from a token digest.
A transactional claim accepts one report for one live runner.
There is no independent expiry before that runner ends.
Old keys stay while uncertain creates or live sandboxes reference them.
Per-month runtime observations keep incomplete costs explicit.
Each delivery separates its saved start-fee bound from a confirmed fee.
Spend and savings select the confirmed fee, or else the saved fee bound.
The selected fee does not become a provider-confirmed charge.
Runtime bounds never supply measured runtime facts.
Running observations and total runtime costs survive a failed monthly read.
The overflow service allocates confirmed monthly costs or conservative bounds.
The store does not turn a cross-month estimate into a measured price interval.
Measured billing coverage stays separate from lifecycle end and its proven
provider lifetime bound.
Finite measured intervals clamp to their latest provider observation.
One known-sandbox value contains both identity and lifetime proof.
The overflow service supplies prices, calculations, KMS, and provider calls.
