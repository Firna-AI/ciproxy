// The pages that show in place of the dashboard for a session without a
// workspace: the App install, an install on a personal account, a removed
// App, and an ended access. Each page shows only for the session state that
// the landing sends to it; any other state goes back to the landing.

import { useState } from "react";

import type { SessionView } from "../api/schema.ts";
import { AuthPage } from "../components/auth-page.tsx";
import { SessionWaiting } from "../components/page-state.tsx";
import { pages } from "../routing/pages.ts";
import { useCheckAgain, useRedirect, useSession } from "../services/hooks.ts";
import {
  accessEndedPage,
  appRemovedPage,
  installPage,
  personalAccountPage,
} from "../view/access.ts";
import type { AuthPageModel } from "../view/page-model.ts";

function StatePage({
  page,
  model,
}: {
  page: string;
  /** The model of the page, or null when the session belongs elsewhere. */
  model: (session: SessionView) => AuthPageModel | null;
}) {
  const snapshot = useSession(page);
  const checkAgain = useCheckAgain();
  const [checking, setChecking] = useState(false);
  const current =
    snapshot.status === "signed-in" ? model(snapshot.session) : null;
  useRedirect(
    snapshot.status === "signed-in" && current === null ? pages.home : null,
  );
  if (snapshot.status !== "signed-in" || current === null) {
    return <SessionWaiting snapshot={snapshot} />;
  }
  const session = snapshot.session;
  return (
    <AuthPage
      model={current}
      handlers={{
        busy: checking ? "check-again" : null,
        onCheckAgain: () => {
          if (checking) return;
          setChecking(true);
          void checkAgain(session).finally(() => setChecking(false));
        },
      }}
    />
  );
}

export function InstallScreen() {
  return (
    <StatePage
      page="install"
      model={(session) =>
        session.state === "install_required" || session.workspaces.length === 0
          ? installPage(session)
          : null
      }
    />
  );
}

export function PersonalAccountScreen() {
  return (
    <StatePage
      page="personal-account"
      model={(session) =>
        session.setup?.kind === "organizations_only"
          ? personalAccountPage(session, session.setup.remove_url)
          : null
      }
    />
  );
}

export function AppRemovedScreen() {
  return (
    <StatePage
      page="app-removed"
      model={(session) =>
        session.state === "app_removed"
          ? appRemovedPage(session, session.removed_organization ?? null)
          : null
      }
    />
  );
}

export function AccessEndedScreen() {
  return (
    <StatePage
      page="access-ended"
      model={(session) =>
        session.state === "access_ended" ? accessEndedPage(session) : null
      }
    />
  );
}
