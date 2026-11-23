# Two smoke steps judge the swap apart

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-23, from the branch `calummoore/swap-support-3dll`,
  review finding 2, the wider fix.
- **Source:** Review.
- **Category:** Test.
- **Location:** `image/smoke-workflow.yml`,
  `crates/ciproxy-agent/scripts/github-smoke-workflow.yml`,
  `image/ciproxy-swap.sh`, and `image/check-swap.py`.
- **Reason:** The direct fix went into the branch, and a verifier in the
  image changes how both smoke workflows get their check.

- **Terms:**
  - **zram:** a Linux block device that keeps its data compressed in
    memory. Each Linux job gets a zram swap from `image/ciproxy-swap.sh`.
  - **Smoke step:** the first step of the Boat image smoke and the `swap`
    mode of the agent smoke. Each checks that a zram swap of the expected
    size is active in a live job.
- **Background:**
  - The swap helper decides at launch whether the kernel has zram and
    whether the swap table exists. It warns and exits 0 when it makes no
    swap.
  - Each smoke step repeats its own probe and its own size rule in inline
    shell and `awk`. The workflows run from copies in private smoke
    repositories, so no offline test covers them.
  - Review finding 2 found that the Boat step failed on a kernel without a
    swap table before its no-zram warning. The branch fixed the order.
- **Problem:** The probe and the size rule live in three places. A later
  change to one can make the smoke disagree with the helper again.
- **Impact if no action:** A smoke step can fail a good release, or pass a
  wrong one, after a change to the helper.
- **Root question:** How do the smoke steps judge the swap by the same
  rules as the helper?
- **Options:**
  - **A.** Keep the inline steps. Effort: Small.
  - **B.** Add a verify mode to `image/ciproxy-swap.sh` that reuses its
    probe, prints the swap table, and checks the size. A flag lets the
    Boat smoke accept a kernel without zram. Cover each outcome in
    `image/check-swap.py`. Both workflows call the mode. Cost: the steps
    need an image with the mode. Effort: Medium.
  - **C.** Let the helper write its outcome to a file, and let the smoke
    print it and act on it. Cost: one more runtime file in both images.
    Effort: Medium.
- **Left out:** A separate shared command outside the image, because the
  workflow copies cannot load it and it gets no offline test.
- **Wider fix:** B.
- **Previous examples:** None found.
- **Recommendation:** B. One script owns the probe and the rule, and the
  offline check covers it.
- **Action:** fix after merge, apply B, because it changes how both smoke
  workflows get their check.
