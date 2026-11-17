# The cost claim prices need a recheck

- **Severity:** Medium. **Effort:** Small. **Confidence:** High.
  **Harder later:** No.
- **Kind:** Post-merge task.
- **Added:** 2026-11-17, from the launch milestone and the post-merge
  list of `plans/ciproxy-mvp.md`, moved to a follow-up item at the user's
  request.
- **Location:** `ts/site/src/data/price-lists.ts`, the `checked` date and
  the amounts. `ts/site/src/components/pricing/SavingCalculation.astro`
  shows them.
- **Reason:** One price needs a person in a browser, and the check repeats
  each month after the launch.

- **Terms:**
  - **Cost claim:** the saving that the site claims for an own machine
    against hosted runners, with the date on which each price was checked.
- **Background:**
  - The claim uses GitHub Linux 2-core at $0.006 a minute, Blacksmith
    2 vCPU at $0.004 a minute, and a Hetzner AX42 at $119.00 a month with
    $59.00 to set up, in the US dollar view of Hetzner.
  - The site gives the check date as 4 November 2026.
  - A check on 16 November 2026 found GitHub and Blacksmith unchanged.
    GitHub's billing docs still list self-hosted runners as free.
  - GitHub postponed a charge of $0.002 a minute for self-hosted runners
    and gave no new date. That charge would lower the claim to about 54%.
  - Hetzner showed the AX42 at €99 a month and €49 to set up. Its US
    dollar view needs a browser, so the date stays until a person confirms
    $119 and $59 there.
  - The orchestrator's call of 4 November 2026: the claim names its date,
    so a monthly check keeps it true.
- **Task:**
  1. Before the launch, a person confirms the Hetzner AX42 prices in the
     US dollar view of the Hetzner site.
  2. Check the GitHub and Blacksmith prices and the self-hosted runner
     charge again.
  3. Update the amounts and the `checked` date in `price-lists.ts`.
  4. After the launch, repeat the check each month.
- **Impact if no action:** The site can show a wrong saving, and a wrong
  price claim harms trust.
- **Needs:** A person with a browser for the Hetzner US dollar view.
- **Done when:** The site shows prices confirmed before the launch with
  their date, and a monthly check has an owner.
- **Related:** `site-and-app-publication.md` waits for the check before
  the launch.
