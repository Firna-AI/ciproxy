# Lawful bases sit in free prose

- **Severity:** Low. **Effort:** Medium. **Confidence:** Medium.
  **Harder later:** No.
- **Kind:** Fix after merge.
- **Added:** 2026-11-22, from the branch `docs/legal-pages-firna`, the
  split-off wider fix of review finding 3.
- **Location:** `ts/site/src/data/legal-privacy.ts`, "How ciproxy uses
  data". `ts/site/src/data/legal.ts`. `ts/site/src/components/legal/`.
  `docs/mockups/src/screens/marketing/legal-page.tsx` and
  `docs/mockups/src/fixtures/legal.ts`.
- **Reason:** The fix needs a mockup change and a new part of the legal
  page, which go beyond the legal text change.

- **Terms:**
  - **Lawful basis:** the legal reason under the UK GDPR for one use of
    personal data, such as a contract or a legitimate interest.
- **Background:**
  - The privacy policy names a lawful basis in the prose of each use.
  - The first text named the contract with the customer for people outside
    that contract, such as the staff of a customer. The review found it, and
    the text now explains that the contract applies only to the customer.
  - The basis decides the rights: the right to object applies to legitimate
    interests, and data portability applies to the contract.
- **Problem:** Nothing stops a new use from naming a wrong basis for a group
  of people.
- **Impact if no action:** A later change can give people the wrong basis and
  the wrong rights again.
- **Root question:** How does each use of personal data name a basis for each
  group of people, so that a wrong basis cannot be written?
- **Options:**
  - **A.** Keep the bases in prose. Effort: Small.
  - **B.** Put the bases in a typed table in `ts/site/src/data/legal.ts`.
    Each row is one use, with the basis for the customer, the basis for other
    people, and the interest for each legitimate interest. The type allows
    the contract only for the customer. Update the legal mockup first, then
    the page and the site README. Effort: Medium.
- **Left out:** A test of the prose stays out, because the testing skill
  gives doc prose no test.
- **Wider fix:** B. The type cannot hold the contract for people outside it.
- **Previous examples:** Review finding 3 of the branch
  `docs/legal-pages-firna`.
- **Recommendation:** B. It removes the class of mistake.
- **Action:** fix after merge, apply B, because it needs a mockup change and
  a page part outside the legal text change. The new table is a new version
  of the policy and needs the owner's approval.
