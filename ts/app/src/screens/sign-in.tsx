// Sign-in with GitHub. The page needs no session, so the export renders it
// in full, and its link works before the page script runs. The API sends a
// failed sign-in back with `reason=github`; the app adds `reason=ended` when
// the 30-day sign-in ended. The footer shows the terms notice that the terms
// step shares.

import { useLocalSearchParams } from "expo-router";

import { AuthPage } from "../components/auth-page.tsx";
import { TermsNotice } from "../components/terms-notice.tsx";
import { useMounted } from "../components/viewport.ts";
import { signInReasonParam } from "../routing/pages.ts";
import { signInPage } from "../view/access.ts";

export function SignInScreen() {
  const params = useLocalSearchParams<{ reason?: string | string[] }>();
  // The export renders the page without a query, so the notice of the
  // reason shows after the page mounts, and hydration matches.
  const mounted = useMounted();
  const reason = mounted ? signInReasonParam(params.reason) : null;
  return <AuthPage model={signInPage(reason)} footer={<TermsNotice />} />;
}
