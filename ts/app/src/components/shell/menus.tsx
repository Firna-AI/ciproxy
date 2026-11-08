// The two menus of the top bar. The workspace switch lists every workspace
// of the person, also one with no role yet, with Add an organization for an
// Admin and the install on another organization. The account menu shows the
// person and the role, the claim of the Admin role for an owner, the docs,
// and Sign out. Both are Firna menus, so the keyboard opens and moves them.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";
import { Avatar } from "@firna/ui/avatar";
import { Badge } from "@firna/ui/badge";
import { Button } from "@firna/ui/button";
import { DropdownMenu, type DropdownListEntry } from "@firna/ui/dropdown";

import { colors } from "@ciproxy/tokens";

import type {
  SessionView,
  WorkspaceRole,
  WorkspaceView,
} from "../../api/schema.ts";
import { useServices } from "../../services/context.tsx";
import { initial } from "../../view/format.ts";
import { siteLinks } from "../../view/page-model.ts";
import { accountMenu, workspaceMenu } from "../../view/shell.ts";
import { ClaimIcon, DocsIcon, PlusIcon, SignOutIcon } from "../icons.tsx";
import { text } from "../text.ts";

export function WorkspaceSwitch({
  session,
  workspace,
  onAddOrganization,
  trigger,
}: {
  session: SessionView;
  workspace: WorkspaceView;
  onAddOrganization: () => void;
  /** The visible trigger: the name of the workspace with its mark. */
  trigger: ReactNode;
}) {
  const services = useServices();
  const menu = workspaceMenu(session, workspace);
  const entries: DropdownListEntry[] = [
    { id: "workspaces", label: "Workspaces", type: "section" },
    ...menu.choices.map((choice): DropdownListEntry => ({
      id: `workspace-${choice.id}`,
      label: choice.name,
      secondary: choice.detail,
      selected: choice.current,
      type: "item",
      onPress: () => services.navigator.go(choice.href),
    })),
    { id: "actions", label: "", type: "divider" },
  ];
  if (menu.canAddOrganization) {
    entries.push({
      id: "add-organization",
      label: "Add an organization",
      type: "footer",
      leading: ({ color }) => <PlusIcon color={color} />,
      onPress: onAddOrganization,
    });
  }
  entries.push({
    id: "install",
    label: "Install on another organization",
    type: "footer",
    onPress: () => {
      services.markInstallStarted();
      services.navigator.leave(menu.installUrl);
    },
  });
  return (
    <DropdownMenu
      entries={entries}
      accessibilityLabel="Workspaces"
      align="start"
      style={styles.switchAnchor}
      minWidth={280}
    >
      <Button
        tone="plain"
        accessibilityLabel={`Workspace ${workspace.name}. Switch workspace`}
        content={trigger}
        style={styles.switchButton}
      />
    </DropdownMenu>
  );
}

export function AccountMenu({
  session,
  workspace,
  role,
  onClaim,
  onSignOut,
  size,
}: {
  session: SessionView;
  workspace: WorkspaceView;
  role: WorkspaceRole;
  onClaim: () => void;
  onSignOut: () => void;
  size: number;
}) {
  const services = useServices();
  const model = accountMenu(session, workspace, role);
  const entries: DropdownListEntry[] = [];
  if (model.canClaim) {
    entries.push({
      id: "claim",
      label: "Claim the Admin role",
      secondary: "You own an organization of this workspace on GitHub.",
      type: "item",
      leading: ({ color }) => <ClaimIcon color={color} />,
      onPress: onClaim,
    });
  }
  entries.push(
    {
      id: "docs",
      label: "Docs",
      type: "item",
      leading: ({ color }) => <DocsIcon color={color} />,
      onPress: () => services.navigator.leave(siteLinks.docs),
    },
    {
      id: "sign-out",
      label: "Sign out",
      type: "item",
      leading: ({ color }) => <SignOutIcon color={color} />,
      onPress: onSignOut,
    },
  );
  const header = (
    <View style={styles.header}>
      <View style={styles.person}>
        <Avatar label={initial(model.login)} size={32} tone="soft" decorative />
        <Text style={[text.body, text.strong]}>{model.login}</Text>
      </View>
      <View style={styles.role}>
        <Text style={text.small}>Role in {model.workspaceName}</Text>
        <Badge tone="neutral" variant="outline">
          {model.roleName}
        </Badge>
      </View>
    </View>
  );
  return (
    <DropdownMenu
      entries={entries}
      header={header}
      accessibilityLabel="Account"
      align="end"
      style={styles.accountAnchor}
      minWidth={280}
    >
      <Button
        tone="plain"
        shape="circle"
        boxSize={size + 8}
        accessibilityLabel={`Account of ${model.login}`}
        content={
          <Avatar
            label={initial(model.login)}
            size={size}
            tone="soft"
            decorative
          />
        }
      />
    </DropdownMenu>
  );
}

// A long workspace name shrinks the switch, never the account: the switch,
// its button, and its trigger may shrink, and the name ends with an ellipsis.
// The accessible name of the switch keeps the whole name.
const styles = StyleSheet.create({
  switchAnchor: { alignSelf: "center", flexShrink: 1, minWidth: 0 },
  switchButton: { flexShrink: 1, minWidth: 0, maxWidth: "100%" },
  accountAnchor: { alignSelf: "center", flexShrink: 0 },
  header: {
    gap: 10,
    paddingVertical: 12,
    paddingHorizontal: 14,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
  },
  person: { flexDirection: "row", alignItems: "center", gap: 10 },
  role: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
  },
});
