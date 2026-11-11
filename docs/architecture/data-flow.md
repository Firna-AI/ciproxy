# Data flow

This page belongs to the [architecture](../architecture.md). It follows
each main flow of data across the
[boundaries](../architecture.md#boundaries) of the components.

## A job on an own machine

1. GitHub queues a job whose `runs-on` names a ciproxy label. It sends a
   message about the job to the control plane.
2. GitHub gives the job to a ready warm runner of its organization and
   label, and the job starts at once. Without a warm runner, the scaler
   starts a runner in free [slots](../protocol/job-routing.md#slots) of an
   own machine of the workspace.
3. The control plane reads the ref, the event, and the head repository of
   the run of the job from the GitHub API. It sends them to the agent for
   the [cache scopes](../protocol/cache.md#cache-scopes) and the Docker disk
   of the job.
4. Metering counts one start. When a slot and its disk are free, the
   scaler sends the agent a registration for a new warm runner.
5. When the job ends, the agent ends the job VM and reports it.

## A job on Boat

1. GitHub queues a job of a ciproxy label. No own machine has free slots
   for it, so the job waits in GitHub.
2. When the job has waited the
   [overflow wait](../protocol/job-routing.md#where-a-job-runs), the scaler
   starts a runner on Boat. The backend client starts its sandbox from the
   image in the customer's Boat account.
3. GitHub gives a waiting job of the label to the new runner. Metering
   counts one start.
   [Overflow runners](../protocol/overflow.md#overflow-runners) says what a
   Boat runner without a job does.
4. When the job ends, the backend client deletes the sandbox. It adds the
   cost of the sandbox to the Boat spend of the month.

## A cache restore and save

1. On an own machine, a step such as `actions/cache` restores an entry. The
   [cache scope](../protocol/cache.md#cache-scopes) of the job decides
   which entries it can restore.
2. The agent gets the current [entry facts](../protocol/cache.md#terms) of
   the repository from the control plane. The control plane reads them from
   GitHub, and it answers only the machine that runs the job.
3. The machine cache picks the first match of the
   [restore order](../protocol/cache.md#restore-order) across both caches.
   It gives the contents when the match is its own. Otherwise they come
   from the GitHub cache, and the machine cache keeps them with their entry
   facts. When the machine cache fails, the job falls back to the
   [GitHub cache](../protocol/cache.md#fallback-to-the-github-cache).
4. A save goes to the machine cache and through to the GitHub cache.
   [Trusted write-back](../protocol/cache.md#trusted-write-back) says which
   saves other branches can restore.
5. A job on Boat restores and saves through the GitHub cache only.
6. The agent reports the cache hits of its machine to the control plane.
   It also reports the [cache counts](../protocol/cache.md#terms) of each
   job whose restores the machine cache handled. A sandbox reports no cache
   counts. The cache contents stay on the machine and in the GitHub cache.

## A checkout and a failed payment

1. In the dashboard, the customer picks a tier and the add-on quantities.
   Billing opens [Stripe Checkout](../protocol/billing.md#checkout), where
   the customer pays.
2. Stripe sends an event that confirms the payment. Billing then changes
   the tier, and a new [period](../protocol/metering.md#periods) starts.
3. When a renewal charge fails, Stripe sends an event and tries again
   later. [Failed payments](../protocol/billing.md#failed-payments) says
   what the dashboard shows.
4. The customer changes the card in the Stripe customer portal. Stripe
   sends an event for each later charge.
5. After the last failed retry, Stripe ends the subscription. Billing moves
   the workspace to Free.

## Machine reports and offline detection

1. The agent registers once with the
   [install token](../protocol/agent.md#install). Then it connects out with
   its machine credential.
2. At a fixed interval, the agent reports its slots, its job VMs, its warm
   runners, its [health](../protocol/agent.md#health) checks, and its
   version.
3. When no report arrives for a
   [set time](../protocol/agent.md#connection), the control plane marks the
   machine offline. The scaler starts no new runner on it.
   [Failures](../protocol/job-routing.md#failures) says what happens to its
   jobs.
4. When the agent connects again, the machine is back online.

## A control plane outage

1. Running jobs go on, because each runner talks to GitHub directly.
2. No new runner starts on an own machine or on Boat. Jobs without a runner
   stay queued in GitHub. [Failures](../protocol/job-routing.md#failures)
   holds the rules.
3. The agents keep their job VMs and warm runners, and they try to connect
   again.
4. When the control plane is back, the scaler starts runners for the queued
   jobs. The backend client deletes each sandbox that ciproxy started, that
   runs no job, and that no waiting job needs.
