// The Members tab: the requests for access that wait for an Admin, the
// members and their roles, and what each role may do. A pick in a role
// picker saves at once, except a pick of Admin, which shows the trust dialog
// first. One change runs at a time, and a failed change says so above the
// lists.

import { useEffect, useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import type { ApiResult } from "../../api/client.ts";
import type { Success, WorkspaceRole } from "../../api/schema.ts";
import type { MembersView } from "../../api/settings-schema.ts";
import { Panel } from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import type { MembersStore } from "../../state/settings-members.ts";
import { settingsServices } from "../../state/settings-services.ts";
import type { FailureCopy } from "../../view/claim.ts";
import { joinWords, roleNames } from "../../view/format.ts";
import {
  memberFailure,
  membersModel,
  requestRole,
  roleDescriptions,
  roleOrder,
  roleStep,
  type MemberAction,
} from "../../view/settings-members.ts";
import type { WorkspaceContext } from "../workspace-page.tsx";
import { GrantAdminDialog } from "./grant-admin-dialog.tsx";
import { MemberList } from "./member-rows.tsx";
import { RequestItem } from "./request-row.tsx";
import { SettingsRead } from "./settings-read.tsx";

type Busy = { id: string; action: MemberAction; role: WorkspaceRole | null };
type Granting = { id: string; name: string; action: "role" | "grant" };

function Members({
  context,
  store,
  view,
  organizations,
}: {
  context: WorkspaceContext;
  store: MembersStore;
  view: MembersView;
  organizations: readonly string[];
}) {
  const phone = useViewport() === "mobile";
  const model = membersModel(view, context.session.login);
  const [busy, setBusy] = useState<Busy | null>(null);
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const [picked, setPicked] = useState<Readonly<Record<string, WorkspaceRole>>>(
    {},
  );
  const [granting, setGranting] = useState<Granting | null>(null);

  const run = async (
    next: Busy,
    work: () => Promise<ApiResult<Success>>,
  ): Promise<void> => {
    setBusy(next);
    setFailure(null);
    const result = await work();
    setBusy(null);
    if (!result.ok) setFailure(memberFailure(result.failure, next.action));
  };

  const give = (
    id: string,
    name: string,
    current: WorkspaceRole | null,
    role: WorkspaceRole,
  ) => {
    const action = current === null ? "grant" : "role";
    const step = roleStep(current, role);
    if (step === "confirm-admin") setGranting({ id, name, action });
    if (step === "save")
      void run({ id, action, role }, () => store.saveRole(id, role));
  };

  return (
    <>
      {failure ? (
        <Notice tone="danger" title={failure.title}>
          {failure.body}
        </Notice>
      ) : null}
      {model.requests.length > 0 ? (
        <Panel title="Asked for access">
          {model.requests.map((row) => {
            const role = picked[row.id] ?? requestRole;
            const mine = busy?.id === row.id ? busy.action : null;
            return (
              <RequestItem
                key={row.id}
                row={row}
                role={role}
                busy={mine === "grant" || mine === "decline" ? mine : null}
                disabled={busy !== null}
                onPick={(next) => setPicked({ ...picked, [row.id]: next })}
                onGrant={() => give(row.id, row.name, null, role)}
                onDecline={() =>
                  void run({ id: row.id, action: "decline", role: null }, () =>
                    store.remove(row.id),
                  )
                }
              />
            );
          })}
        </Panel>
      ) : null}
      <Panel title="Members" flush>
        <MemberList
          rows={model.members}
          phone={phone}
          handlers={{
            pending:
              busy !== null &&
              (busy.action === "role" || busy.action === "remove")
                ? { id: busy.id, role: busy.role }
                : null,
            onPick: (row, role) => give(row.id, row.name, row.role, role),
            onRemove: (row) =>
              void run({ id: row.id, action: "remove", role: null }, () =>
                store.remove(row.id),
              ),
          }}
        />
      </Panel>
      <Text style={text.caption}>
        {organizations.length > 0
          ? `Other members of ${joinWords(organizations, "and")} on GitHub ask for access when they sign in.`
          : "Other members of your organizations on GitHub ask for access when they sign in."}
      </Text>
      <Panel title="Roles">
        {roleOrder.map((role) => (
          <View key={role} style={styles.role}>
            <Text style={[text.small, text.strong, styles.roleName]}>
              {roleNames[role]}
            </Text>
            <Text style={[text.small, styles.roleText]}>
              {roleDescriptions[role]}
            </Text>
          </View>
        ))}
      </Panel>
      {granting ? (
        <GrantAdminDialog
          name={granting.name}
          notice={view.admin_notice}
          action={granting.action}
          grant={() => store.saveRole(granting.id, "admin")}
          onClose={() => setGranting(null)}
        />
      ) : null}
    </>
  );
}

export function MembersTab({ context }: { context: WorkspaceContext }) {
  const services = settingsServices(useServices());
  const store = services.members(context.workspace.id);
  const snapshot = useValue(store.state);
  const settings = useValue(services.settings(context.workspace.id).state);
  useEffect(() => {
    void store.load();
  }, [store]);
  const organizations =
    settings.status === "ready"
      ? settings.value.organizations
          .filter((item) => item.state !== "removed")
          .map((item) => item.login)
      : [];
  return (
    <SettingsRead
      snapshot={snapshot}
      subject="members"
      onRetry={() => void store.load()}
      loading={
        <Panel title="Members" flush>
          <MemberList rows={[]} phone={false} handlers={idle} loading />
        </Panel>
      }
    >
      {(view) => (
        <Members
          context={context}
          store={store}
          view={view}
          organizations={organizations}
        />
      )}
    </SettingsRead>
  );
}

const idle = {
  pending: null,
  onPick: () => undefined,
  onRemove: () => undefined,
};

const styles = StyleSheet.create({
  role: { flexDirection: "row", flexWrap: "wrap", gap: 12 },
  roleName: { width: 72 },
  roleText: { flexGrow: 1, flexShrink: 1, flexBasis: 220 },
});
