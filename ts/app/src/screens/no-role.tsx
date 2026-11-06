// The page that a person with no role sees in place of every page of a
// workspace. It names the Admins, checks again, and lets an owner claim the
// Admin role. Links lead to the other workspaces of the person.

import { useState } from "react";
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
  const [busy, setBusy] = useState<PageAction["kind"] | null>(null);
  const [failed, setFailed] = useState(false);
  const others = session.workspaces.filter((item) => item.id !== workspace.id);

  const run = async (
    kind: PageAction["kind"],
    work: () => Promise<boolean>,
  ) => {
    if (busy !== null) return;
    setBusy(kind);
    setFailed(false);
    const done = await work();
    setBusy(null);
    if (!done) setFailed(true);
  };

  return (
    <AuthPage
      model={noRolePage(session, workspace)}
      failure={
        failed
          ? {
              title: "Unable to claim the Admin role",
              body: "GitHub must list you as an owner. Check again after an owner adds you.",
            }
          : null
      }
      handlers={{
        busy,
        onCheckAgain: () =>
          void run("check-again", async () => {
            await checkAgain(session);
            return true;
          }),
        onClaim: (workspaceId) =>
          void run("claim-admin", async () => {
            const result = await services.session.claimAdmin(workspaceId);
            if (result.ok)
              services.navigator.go(workspaceHomeHref(workspaceId));
            return result.ok;
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
