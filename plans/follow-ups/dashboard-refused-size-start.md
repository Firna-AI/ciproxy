# Dashboard ignores usable sizes after a refusal

- **Severity:** Medium. **Likelihood:** Rare. **Effort:** Medium.
  **Confidence:** High. **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch
  `calummoore/ciproxy-providers-interface-s8rp`, review finding 6.
- **Location:** `crates/ciproxy-api/src/handlers/label_provider.rs:45`,
  `crates/ciproxy-api/src/handlers/label_routing.rs:86`,
  `crates/ciproxy-api/src/wire/job_fit.rs` (`ProviderLabelFit`), and
  `ts/app/src/view/jobs-places.ts:204`.
- **Reason:** The case is rare, because the provider's capacity read
  already leaves out sizes that the account cannot use, and each
  five-minute catalog refresh clears the refused sizes.

- **Source:** Review.
- **Category:** Product bug.
- **Terms:**
  - **Size refusal:** a provider declined starts on one VM size. Only that
    size stops fitting, as `docs/protocol/overflow.md` says.
  - **Largest-size fallback:** "Run on the largest size" starts a job on the
    largest provider size that holds the image when no place fits.
- **Background:**
  - The scaler picks the cheapest fitting size from the catalog without the
    refused sizes, and for the fallback it takes the largest of those.
  - The dashboard API sends each provider's physical fit, with the size
    name and a `refused` flag, beside an `eligible_size`.
  - Jobs reads the physical size and its `refused` flag, not
    `eligible_size`. For the fallback, the label API checks the largest
    size of the unfiltered catalog.
- **Problem:** After a provider refuses one size, Jobs can say that a label
  waits while the scaler starts it on another size. The label view can say
  that a largest-size label waits while the scaler sends it to the next
  size.
- **Impact if no action:** In this rare window, the dashboard shows a wait
  that does not happen, until the next catalog refresh.
- **Root question:** How do Labels and Jobs say whether, and on which size,
  a provider starts a label's jobs after it refused some sizes?
- **Options:**
  - **A.** Jobs reads `eligible_size`, and the label API takes the fallback
    from the filtered catalog. Effort: Small.
  - **B.** Share the scaler's size policy with both dashboard views.
    Effort: Medium.
  - **D.** B, and change `ProviderLabelFit`: the physical fit says only
    whether a size fits, and one `start` field gives the usable size from
    the shared policy, marks the fallback, and is empty when no usable size
    remains. Labels, the dialog captions, and Jobs read `start`. Test a
    refused cheapest size and a refused largest size, each with a usable
    replacement. Effort: Medium.
  - **E.** One scaler start decision for each provider, which the label API
    also calls, including the spend admission of review finding 1.
    Effort: Large.
- **Left out:** Hiding every refusal, because `docs/protocol/labels.md`
  needs the refusal as the wait reason when no usable size remains. A
  scaler that waits when the largest size is refused, because it breaks the
  rule that only that size stops fitting.
- **Wider fix:** D. One start field stops each view from reading the
  physical size as the start size.
- **Previous examples:** `b584e079` fixed the selection after refusals.
  `0ade9745` corrected Jobs notices that overstated fit.
- **Recommendation:** D.
- **Action:** fix after merge, apply D, because the likelihood is Rare and
  the severity Medium.
