// The notice of a failed session check. After a session loaded, a later read
// can fail: Check again, a page change, or the read after a claim. The page
// keeps what it shows and its input, and this notice says that the check did
// not finish, with one way to try again. The frames of the app show it, so
// every page that uses the session shows it with no wiring of its own. A
// first read that fails and an ended sign-in keep their own pages.

import { useState } from "react";

import { useServices } from "../services/context.tsx";
import { useValue } from "../services/hooks.ts";
import { ActionButton } from "./actions.tsx";
import { Notice } from "./notice.tsx";

export function SessionRefreshNotice() {
  const { session } = useServices();
  const snapshot = useValue(session.state);
  const [busy, setBusy] = useState(false);
  if (snapshot.status !== "signed-in" || snapshot.refreshFailure === null) {
    return null;
  }
  const retry = () => {
    if (busy) return;
    setBusy(true);
    void session.refresh().finally(() => setBusy(false));
  };
  return (
    <Notice
      tone="warning"
      title="Unable to check your access"
      actions={
        <ActionButton label="Try again" size="sm" busy={busy} onPress={retry} />
      }
    >
      This page shows what ciproxy read last.
    </Notice>
  );
}
