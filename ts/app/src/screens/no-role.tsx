// The page that a person with no role sees in place of every page of a
// workspace. It names the Admins, checks again, and lets an owner claim the
// Admin role. Links lead to the other workspaces of the person.

import { useRef, useState } from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";

import type { SessionView, WorkspaceView } from "../api/schema.ts";
import { ActionLink } from "../components/actions.tsx";
import { AuthPage } from "../components/auth-page.tsx";
import { text } from "../components/text.ts";
import { workspaceHomeHref } from "../routing/pages.ts";
import { useServices } from "../services/context.tsx";
import { useCheckAgain } from "../services/hooks.ts";
import { noRolePage } from "../view/access.ts";
import { claimFailure, type FailureCopy } from "../view/claim.ts";
import type { PageAction } from "../view/page-model.ts";

export function NoRoleScreen({
  session,
  workspace,
}: {
  session: SessionView;
  workspace: WorkspaceView;
}) {
  const services = useServices();
  const checkAgain = useCheckAgain();
  const running = useRef(false);
  const [busy, setBusy] = useState<PageAction["kind"] | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const others = session.workspaces.filter((item) => item.id !== workspace.id);

  // One action at a time. A failed session check shows in the frame; a
  // failed claim shows here, with no cause that the API does not give.
  const run = async (
    kind: PageAction["kind"],
    work: () => Promise<FailureCopy | null>,
  ) => {
    if (running.current) return;
    running.current = true;
    setBusy(kind);
    setFailure(null);
    const next = await work();
    running.current = false;
    setBusy(null);
    setFailure(next);
  };

  return (
    <AuthPage
      model={noRolePage(session, workspace)}
      failure={failure}
      handlers={{
        busy,
        onCheckAgain: () =>
          void run("check-again", async () => {
            await checkAgain(session);
            return null;
          }),
        onClaim: (workspaceId) =>
          void run("claim-admin", async () => {
            const result = await services.session.claimAdmin(workspaceId);
            if (!result.ok) return claimFailure(result.failure);
            services.navigator.go(workspaceHomeHref(workspaceId));
            return null;
          }),
      }}
    >
      {others.length > 0 ? (
        <View style={styles.others}>
          <Text style={text.caption}>Your other workspaces</Text>
          <View style={styles.links}>
            {others.map((item) => (
              <Link key={item.id} href={workspaceHomeHref(item.id)} asChild>
                <ActionLink
                  label={item.name}
                  href={workspaceHomeHref(item.id)}
                  tone="ghost"
                  size="sm"
                />
              </Link>
            ))}
          </View>
        </View>
      ) : null}
    </AuthPage>
  );
}

const styles = StyleSheet.create({
  others: { gap: 6 },
  links: { flexDirection: "row", flexWrap: "wrap", gap: 4, marginLeft: -12 },
});
