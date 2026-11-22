# The terms do not say who binds the customer

- **Severity:** High. **Effort:** Medium. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Needs decision.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, review
  finding 11.
- **Location:** `ts/site/src/data/legal-terms.ts`: the acceptance and
  authority sentences of "The service", the role duties of "Accounts,
  workspaces, and organizations", the price change and machine duties, the
  content duties of "Your data", and "Changes to these terms".
- **Reason:** The answer decides whose acceptance binds the customer, and
  only the user can choose that.

- **Terms:**
  - **Viewer:** the workspace role that can see jobs, machines, savings, and
    Plan and usage, and changes nothing.
- **Background:**
  - The terms say that each person accepts the current version at each
    sign-in with GitHub. The user chose this rule from the conductive terms
    on 22 November 2026.
  - The terms also say: "You must have the authority to accept these terms
    for the customer". A Viewer, a Billing person, or a member who only asks
    for access can lack that authority.
  - The terms define the customer as "the person or organization that holds
    a ciproxy workspace". The terms also use "organization" for a GitHub
    organization, which is not a legal person, and a workspace can hold
    several.
  - The terms use one "you" for the customer and for each person who signs
    in. So the duties of the customer also fall on a Viewer: the roles of
    the workspace, the machine and Boat costs, and the job content.
  - Conductive's terms have no authority clause, but they also have no paid
    plans and no customer party.
- **Problem:** The text does not say whose acceptance binds the customer, and
  it gives each person who signs in the duties of the customer.
- **Impact if no action:** A Viewer can seem to break the terms by signing
  in, and Firna Ltd can mistake a Viewer's sign-in for the customer's
  agreement.
- **Root question:** Which party does each clause of the terms bind, the
  customer or the person who signs in, and which act binds the customer to
  each version?
- **Options:**
  - **A.** Each person accepts for their own use at each sign-in. Only a
    person with authority accepts for the customer. This gives no proof of
    who had authority. Effort: Small.
  - **B.** A separate agreement for each customer, with acceptance at
    sign-in kept for each person. Firna Ltd keeps the customer agreements
    and their changes apart. Effort: Medium.
  - **C.** Record each person's acceptance and the customer's acceptance
    apart in the dashboard. An authorized person accepts for the workspace,
    and ciproxy records the version. Effort: Large.
  - **D.** Define "the customer", with its legal name, and "you", the person
    who signs in, one time, and name the right party in each clause. Each
    person accepts for their own use. The first Admin also accepts for the
    customer, and the customer lets each Admin accept later versions for it.
    The dialog that grants the Admin role names this power, with its mockup
    and a decision in `docs/protocol/security.md`. Effort: Medium.
  - **E.** A or D for the first acceptance. Each later version binds the
    customer from a set date after publication and notice, as
    `terms-changes-unattended-workspaces.md` proposes. Effort: Medium.
  - **F.** Turn on the required terms box of Stripe Checkout, so the person
    who first buys a plan accepts for the customer. Keep A or D for Free
    workspaces and for later versions. Effort: Medium.
- **Left out:** Deleting the authority sentence, as conductive has none,
  fails the goal, because a person who buys a plan for a company would then
  be the only party.
- **Wider fix:** C. One record holds the person, the version, the time, and
  whether the acceptance binds the customer. It also gives the other two
  acceptance items their proof.
- **Previous examples:** None found.
- **Related:**
  - `terms-notice-on-every-sign-in.md`: the same sentence. That item decides
    which sign-ins show the notice. This item decides whose acceptance binds
    the customer. Its option C and this option C are one record.
  - `terms-changes-unattended-workspaces.md`: with A or D, fewer sign-ins
    bind the customer, so a workspace keeps an old version longer. Option E
    here uses its rule.
- **Recommendation:** D. It names who binds the customer, the first Admin and
  then each Admin, and keeps acceptance at each sign-in. Answer it with the
  two related items, because the three set one acceptance rule.
- **Action:** ask the user, because the recorded decision sets when a person
  accepts the terms, not whose acceptance binds the customer.
