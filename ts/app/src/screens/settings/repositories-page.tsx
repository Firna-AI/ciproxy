// The repository access of one organization, which the Organizations tab
// opens: all private repositories, also new ones, or the selected ones.
// GitHub decides which repositories ciproxy can see, so the page links to
// the install on GitHub. A removed organization has no access to show.

import { useEffect } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import type { OrganizationView } from "../../api/schema.ts";
import { ActionLink } from "../../components/actions.tsx";
import { Panel, TextLink } from "../../components/feature-parts.tsx";
import { ExternalIcon, InfoIcon } from "../../components/icons.tsx";
import {
  ReadFailure,
  StateBlock,
  WaitingLine,
} from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { settingsServices } from "../../state/settings-services.ts";
import type { SettingsStore } from "../../state/settings-store.ts";
import { settingsHref } from "../../view/settings-tabs.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { AccessForm } from "./access-form.tsx";
import { SettingsRead } from "./settings-read.tsx";

function Repositories({
  context,
  store,
  organization,
}: {
  context: WorkspaceContext;
  store: SettingsStore;
  organization: OrganizationView;
}) {
  const repositories = settingsServices(useServices()).repositories(
    context.workspace.id,
    organization.id,
  );
  const snapshot = useValue(repositories.state);
  useEffect(() => {
    void repositories.load();
  }, [repositories]);
  return (
    <>
      <Panel title="Repositories that use ciproxy runners">
        {snapshot.status === "loading" ? (
          <WaitingLine>Loading the repositories…</WaitingLine>
        ) : snapshot.status === "failed" ? (
          <ReadFailure
            subject="the repositories"
            onRetry={() => void repositories.load()}
          />
        ) : (
          <AccessForm
            key={JSON.stringify(organization.access)}
            store={store}
            organization={organization}
            repositories={snapshot.repositories}
          />
        )}
      </Panel>
      <Panel title="GitHub App">
        <View style={styles.spread}>
          <Text style={[text.small, styles.grow]}>
            GitHub decides which repositories ciproxy can see. Change them on
            GitHub.
          </Text>
          <ActionLink
            label="Manage on GitHub"
            href={organization.manage_url}
            size="sm"
            icon={ExternalIcon}
            newTab
          />
        </View>
      </Panel>
    </>
  );
}

export function RepositoriesPage({
  context,
  organizationId,
}: {
  context: WorkspaceContext;
  organizationId: string;
}) {
  const store = settingsServices(useServices()).settings(context.workspace.id);
  const snapshot = useValue(store.state);
  const back = settingsHref(context.workspace.id, "organizations");
  return (
    <SettingsRead
      snapshot={snapshot}
      subject="settings"
      onRetry={() => void store.load()}
      loading={<WaitingLine>Loading the organization…</WaitingLine>}
    >
      {(settings) => {
        const organization = settings.organizations.find(
          (item) => item.id === organizationId && item.state !== "removed",
        );
        return (
          <>
            <View style={styles.crumbs}>
              <TextLink href={back} label="Organizations" muted />
              <Text style={[text.small, styles.muted]}>/</Text>
              <Text style={[text.small, text.strong]}>
                {organization?.login ?? "Organization"}
              </Text>
            </View>
            {organization === undefined ? (
              <StateBlock
                icon={InfoIcon}
                title="This organization is not in the workspace"
              >
                Open Organizations to see the organizations of this workspace.
              </StateBlock>
            ) : (
              <Repositories
                context={context}
                store={store}
                organization={organization}
              />
            )}
          </>
        );
      }}
    </SettingsRead>
  );
}

const styles = StyleSheet.create({
  spread: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  grow: { flexGrow: 1, flexShrink: 1, flexBasis: 240 },
  crumbs: { flexDirection: "row", alignItems: "center", gap: 6 },
  muted: { color: colors.muted },
});
