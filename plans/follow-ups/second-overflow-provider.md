# Add a second real overflow provider

- **Severity:** Medium. **Effort:** Large. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, the user decision on the
  scope of a second provider.
- **Location:** `crates/ciproxy-overflow/README.md`, section "Add a
  provider", once the provider interface merges.
- **Reason:** On 23 November 2026 the user chose to ship the provider
  interface first, and to add the second real provider in its own plan and
  branch after that merge.

- **Terms:**
  - **Overflow provider:** a cloud service that runs overflow runners in the
    customer's own account while the customer's own machines are busy. Boat
    is the first provider.
  - **Label order:** the ordered list of places where the jobs of one label
    run: own machines and each connected provider.
  - **instabox:** the sandbox provider at instabox.sh. Its API is at
    `https://instabox.sh/docs/api.md`.
- **Background:**
  - The provider interface lets a workspace connect several providers. Each
    label sets its order, and a job goes to the next place when the current
    place cannot take it now. Each provider has its own spend limit. A
    newly offered provider joins the end of each Linux label's order. The
    user decided these rules on 23 November 2026.
  - Boat is the only real provider. The scaler tests cover an order of two
    providers through opaque IDs. The store, the overflow service, the
    dashboard API, and the app have no tests with two providers, because
    the closed provider enum has one value.
  - The mockups show two-provider states with a fixture named "Second
    provider".
  - On 22 November 2026 the user asked to add instabox as a provider. Its
    plan is `plans/overflow-providers-and-instabox.md` on the unmerged
    branch `calummoore/instaboxsh-ci-provider-plan`. That plan also holds
    the provider interface, which is now on `main`, so only its instabox
    parts stay.
  - instabox facts from that plan, read on 22 November 2026:
    - It has no start fee. It bills runtime by the second.
    - A sandbox has a fixed root and a `/data` disk that grows online.
    - A snapshot belongs to one instabox workspace, so each customer
      account needs its own image build.
    - Egress beyond the plan's free amount costs $0.02 per GiB, which the
      full-lifetime spend bound cannot cover.
    - A sandbox sends at 50 Mbit/s after a burst, or 1000 Mbit/s for a `ci`
      sandbox of a paid workspace.
    - A Free workspace cannot create a `ci` sandbox from the API.
  - The user decided two instabox items on 22 November 2026: the customer
    brings its own instabox key, and the first step after the key is saved
    builds the image in the customer's instabox workspace.
  - Three instabox questions are still open: how ciproxy gets a credential
    that can run `ci` sandboxes, how the spend limit counts egress, and how
    customers avoid or see slow uploads.
- **Task:** Write a plan for the second real provider, instabox unless the
  user names another, from the instabox parts of the 22 November plan.
  Follow "Add a provider" in `crates/ciproxy-overflow/README.md`: the
  adapter in `futex-ai/sandbox`, the enum value, the descriptor, the seams,
  the image, the connect form, the protocol sections, the site guide, and
  the live probes. Add the tests with two providers to the store, the
  overflow service, the dashboard API, and the app. Replace the "Second
  provider" mockup fixture with the real provider. Ask the three open
  instabox questions first.
- **Impact if no action:** Customers can connect only Boat. The shared code
  for several providers runs with Boat tests only, so a defect in it shows
  up only when a second provider arrives.
- **Needs:** the provider interface on `main`, a paid instabox workspace
  with an API key, and the user's answers to the three open questions.
- **Done when:** a customer can connect the second provider, put it in a
  label order, and run a job on it, and the store, service, API, and app
  tests cover a workspace with two providers.
