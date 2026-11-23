// The terms step at `/continue`: the page before each GitHub sign-in that does
// not start on sign-in. The page needs no session, so the export renders it in
// full, and its link works before the page script runs. The footer shows the
// terms notice of sign-in.

import { AuthPage } from "../components/auth-page.tsx";
import { TermsNotice } from "../components/terms-notice.tsx";
import { termsStepPage } from "../view/access.ts";

export function TermsStepScreen() {
  return <AuthPage model={termsStepPage()} footer={<TermsNotice />} />;
}
