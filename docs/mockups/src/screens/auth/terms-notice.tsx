// The terms notice of each page that starts a GitHub sign-in: sign-in and the
// terms step. A person accepts the current terms at each sign-in with GitHub,
// so both pages show these same words.

import { TextLink } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";

export function TermsNotice() {
  return (
    <>
      By signing in, you agree to the ciproxy{" "}
      <TextLink to={paths.terms}>Terms</TextLink> and{" "}
      <TextLink to={paths.privacy}>Privacy policy</TextLink>.
    </>
  );
}
