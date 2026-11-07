// The return page of an install on GitHub. It shows each link of the person
// that still needs something, each link that the latest sign-in names, and
// keeps a link that the person acted on here until it ends, so the outcome
// shows. Two links stay independent: each has its own pick, request, and
// failure. A link to another page of the app is the person's exit, which
// ends the continuation of the current install.

import { Fragment, useEffect, useState } from "react";
import { StyleSheet, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import { AuthShell } from "../../components/auth-shell.tsx";
import { SessionWaiting } from "../../components/page-state.tsx";
import { endedContinuationKey } from "../../routing/claim-continuation.ts";
import { verifiedInstallation } from "../../routing/continuation.ts";
import { pages } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { useRedirect, useSession, useValue } from "../../services/hooks.ts";
import { createLinksStore, joinsReturnPage } from "../../state/links.ts";
import { linkSection } from "../../view/add-organization.ts";
import { linkFailure, requestSection } from "../../view/link-notes.ts";
import { checkAgainTarget } from "../../view/access.ts";
import { signedInAs } from "../../view/format.ts";
import { LinkSectionView } from "./link-section.tsx";

export function AddOrganizationScreen() {
  const services = useServices();
  const snapshot = useSession("add-organization");
  const [store] = useState(() =>
    createLinksStore({
      endpoints: services.endpoints,
      session: services.session,
      timers: services.env.timers,
      visibility: services.env.visibility,
    }),
  );
  const links = useValue(store.state);
  useEffect(() => store.watch(), [store]);
  useEffect(() => {
    if (snapshot.status !== "signed-in") return;
    const { session } = snapshot;
    store.seed(session.organization_links, session.installation_continuation);
  }, [snapshot, store]);

  const session = snapshot.status === "signed-in" ? snapshot.session : null;
  const requests = session === null ? null : requestSection(session);
  // The store seeds after the first render, so the links of the session that
  // join the page count too; otherwise the page would leave before it shows
  // them.
  const empty =
    session !== null &&
    links.entries.length === 0 &&
    !session.organization_links.some((link) =>
      joinsReturnPage(link, session.installation_continuation),
    ) &&
    requests === null;
  useRedirect(empty ? pages.home : null);
  if (session === null || empty) return <SessionWaiting snapshot={snapshot} />;

  const checkAgain = () => {
    const target = checkAgainTarget(session);
    if ("leave" in target) services.navigator.leave(target.leave);
    else void store.checkAgain();
  };
  // Leaving for a page of the app ends the continuation of the install: the
  // landing no longer opens its workspace in place of the chosen page. The
  // named claims of the sign-in end too once they all ended, by the newest
  // state on this page. While one still waits, a later landing keeps it.
  const exit = () => {
    const installationId = verifiedInstallation(session);
    if (installationId !== null) services.rememberInstallation(installationId);
    const ended = endedContinuationKey(
      session,
      links.entries.map((entry) => entry.link),
    );
    if (ended !== null) services.rememberContinuation(ended);
    services.clearInstallStarted();
  };
  const handlers = {
    busy: links.checking ? ("check-again" as const) : null,
    onCheckAgain: checkAgain,
    onExit: exit,
  };
  const sections = links.entries.map((entry) => ({
    key: entry.link.id,
    section: linkSection(session, entry),
    failure: entry.failure === null ? null : linkFailure(entry.failure),
  }));
  if (requests !== null) {
    sections.push({ key: requests.claimId, section: requests, failure: null });
  }
  const checkFailure =
    links.checkFailure === null ? null : linkFailure(links.checkFailure);

  return (
    <AuthShell footer={signedInAs(session.login)}>
      {sections.map(({ key, section, failure }, index) => (
        <Fragment key={key}>
          {index > 0 ? <View style={styles.divider} /> : null}
          <LinkSectionView
            section={section}
            failure={failure ?? (index === 0 ? checkFailure : null)}
            handlers={handlers}
            onPick={(workspaceId) => store.pick(section.claimId, workspaceId)}
            onSubmit={() => void store.choose(section.claimId)}
          />
        </Fragment>
      ))}
    </AuthShell>
  );
}

const styles = StyleSheet.create({
  divider: { height: 1, backgroundColor: colors.border },
});
