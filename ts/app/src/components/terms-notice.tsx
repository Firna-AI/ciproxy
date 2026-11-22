// The terms notice of each page that starts a GitHub sign-in: sign-in and the
// terms step. A person accepts the current terms at each sign-in with GitHub,
// so both pages show these same words in their footer.

import { siteLinks } from "../view/page-model.ts";
import { InlineLink } from "./inline-link.tsx";

export function TermsNotice() {
  return (
    <>
      By signing in, you agree to the ciproxy{" "}
      <InlineLink label="Terms" href={siteLinks.terms} /> and{" "}
      <InlineLink label="Privacy policy" href={siteLinks.privacy} />.
    </>
  );
}
