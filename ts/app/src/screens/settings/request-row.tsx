// One request for access on the Members tab: the person, the day of the
// request with the organizations that the person was a member of then, the
// role picker with Viewer picked first, Give access, and Decline.

import { StyleSheet, Text, View } from "react-native";

import type { WorkspaceRole } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { text } from "../../components/text.ts";
import type { RequestRow } from "../../view/settings-members.ts";
import { PersonName, RolePicker } from "./member-rows.tsx";

export function RequestItem({
  row,
  role,
  busy,
  disabled,
  onPick,
  onGrant,
  onDecline,
}: {
  row: RequestRow;
  role: WorkspaceRole;
  busy: "grant" | "decline" | null;
  disabled: boolean;
  onPick: (role: WorkspaceRole) => void;
  onGrant: () => void;
  onDecline: () => void;
}) {
  return (
    <View style={styles.request}>
      <View style={styles.requestText}>
        <PersonName person={row} you={false} />
        <Text style={text.caption}>{row.asked}</Text>
      </View>
      <View style={styles.requestActions}>
        <RolePicker
          name={row.name}
          role={role}
          disabled={disabled}
          onPick={onPick}
        />
        <ActionButton
          label="Give access"
          accessibilityLabel={`Give access to ${row.name}`}
          tone="primary"
          size="sm"
          busy={busy === "grant"}
          disabled={disabled}
          onPress={onGrant}
        />
        <ActionButton
          label="Decline"
          accessibilityLabel={`Decline ${row.name}`}
          tone="ghost"
          size="sm"
          busy={busy === "decline"}
          disabled={disabled}
          onPress={onDecline}
        />
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  request: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
  },
  requestText: { flexGrow: 1, flexShrink: 1, flexBasis: 220, gap: 4 },
  requestActions: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 8,
  },
});
