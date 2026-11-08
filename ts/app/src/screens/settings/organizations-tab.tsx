// The Organizations tab: the name of the workspace, and its GitHub
// organizations with their repository access. The workspace owns the plan,
// the members, the machines, the labels, and the Boat account, and each
// machine runs the jobs of every organization. Add an organization opens
// GitHub; on Free, the new organization stays inactive until Team. The last
// linked organization has no Take out.

import { useState } from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";

import type { SettingsView } from "../../api/settings-schema.ts";
import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { Panel } from "../../components/feature-parts.tsx";
import { PlusIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { useShellActions } from "../../components/shell/shell-actions.ts";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import { workspaceHref } from "../../routing/pages.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { settingsServices } from "../../state/settings-services.ts";
import type { SettingsStore } from "../../state/settings-store.ts";
import {
  linkedCount,
  organizationRows,
  type OrganizationRow,
} from "../../view/settings-organizations.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import {
  RenameDialog,
  TakeOutDialog,
  TurnOnDialog,
} from "./organization-dialogs.tsx";
import {
  OrganizationList,
  OrganizationListLoading,
} from "./organization-list.tsx";
import { SettingsRead } from "./settings-read.tsx";

type Opened =
  | { kind: "rename" }
  | { kind: "take-out"; row: OrganizationRow }
  | { kind: "turn-on"; row: OrganizationRow };

function WorkspacePanel({
  name,
  onRename,
}: {
  name: string;
  onRename: () => void;
}) {
  return (
    <Panel title="Workspace">
      <View style={styles.spread}>
        <View style={styles.intro}>
          <Text style={[text.body, text.strong]}>{name}</Text>
          <Text style={text.caption}>
            Its organizations share its plan, members, machines, labels, and
            Boat account. Each machine runs the jobs of every organization.
          </Text>
        </View>
        <ActionButton label="Rename" size="sm" onPress={onRename} />
      </View>
    </Panel>
  );
}

function FreeNotice({ workspaceId }: { workspaceId: string }) {
  const href = workspaceHref("billing", workspaceId);
  return (
    <Notice
      tone="info"
      title="Run more organizations on Team"
      actions={
        <Link href={href} asChild>
          <ActionLink label="Compare plans" href={href} size="sm" />
        </Link>
      }
    >
      Free includes 1 organization. Team and Business run several, on the same
      machines.
    </Notice>
  );
}

function Organizations({
  context,
  store,
  settings,
}: {
  context: WorkspaceContext;
  store: SettingsStore;
  settings: SettingsView;
}) {
  const workspaceId = context.workspace.id;
  const phone = useViewport() === "mobile";
  const shell = useShellActions(workspaceId);
  const [opened, setOpened] = useState<Opened | null>(null);
  const close = () => setOpened(null);
  const turning =
    opened?.kind === "turn-on"
      ? settings.organizations.find((item) => item.id === opened.row.id)
      : undefined;
  const free =
    !settings.can_add_organization &&
    !context.workspace.organization_limit_exceeded &&
    linkedCount(settings.organizations) === 1;
  return (
    <>
      <WorkspacePanel
        name={settings.name}
        onRename={() => setOpened({ kind: "rename" })}
      />
      {free ? <FreeNotice workspaceId={workspaceId} /> : null}
      {shell.failure ? (
        <Notice tone="danger" title={shell.failure.title}>
          {shell.failure.body}
        </Notice>
      ) : null}
      <Panel
        title="Organizations"
        flush
        action={
          <ActionButton
            label="Add an organization"
            size="sm"
            icon={PlusIcon}
            busy={shell.busy === "add-organization"}
            onPress={() => void shell.addOrganization()}
          />
        }
      >
        <OrganizationList
          rows={organizationRows(settings.organizations)}
          phone={phone}
          handlers={{
            workspaceId,
            onTakeOut: (row) => setOpened({ kind: "take-out", row }),
            onTurnOn: (row) => setOpened({ kind: "turn-on", row }),
            onInstallAgain: () => void shell.addOrganization(),
            installing: shell.busy === "add-organization",
          }}
        />
      </Panel>
      <Text style={text.caption}>
        Add an organization opens GitHub, where you pick the organization and
        its repositories.
        {settings.can_add_organization
          ? ""
          : " On Free, it stays inactive until Team."}
      </Text>
      {opened?.kind === "rename" ? (
        <RenameDialog
          name={settings.name}
          rename={store.rename}
          onClose={close}
        />
      ) : null}
      {opened?.kind === "take-out" ? (
        <TakeOutDialog
          login={opened.row.login}
          takeOut={() => store.takeOut(opened.row.id)}
          onClose={close}
        />
      ) : null}
      {turning ? (
        <TurnOnDialog
          organization={turning}
          turnOn={() => store.turnOn(turning.id)}
          onClose={close}
        />
      ) : null}
    </>
  );
}

export function OrganizationsTab({ context }: { context: WorkspaceContext }) {
  const store = settingsServices(useServices()).settings(context.workspace.id);
  const snapshot = useValue(store.state);
  return (
    <SettingsRead
      snapshot={snapshot}
      subject="settings"
      onRetry={() => void store.load()}
      loading={
        <Panel title="Organizations" flush>
          <OrganizationListLoading />
        </Panel>
      }
    >
      {(settings) => (
        <Organizations context={context} store={store} settings={settings} />
      )}
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
  intro: { flexGrow: 1, flexShrink: 1, flexBasis: 260, gap: 2, maxWidth: 640 },
});
