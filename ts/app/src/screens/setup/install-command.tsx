// The one-line install command of the machine step. The store creates one
// command when the step first shows, never on a refresh. The command shows
// with Copy in its head, then its end, what a machine needs, and Revoke
// token. After Revoke token or the end, a new command is one press away. The
// copy result belongs to the text of the command, so a new command never
// shows the result of an earlier one.

import { useEffect } from "react";
import { StyleSheet, Text, View } from "react-native";

import { ActionButton } from "../../components/actions.tsx";
import { CodeBlock } from "../../components/code-block.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { commandExpired } from "../../state/install-command.ts";
import { formatTime } from "../../view/format.ts";

export function InstallCommandView({ workspaceId }: { workspaceId: string }) {
  const services = useServices();
  const store = services.installCommand(workspaceId);
  const snapshot = useValue(store.state);
  useEffect(() => {
    if (store.state.get().status === "none") void store.create();
  }, [store]);

  if (snapshot.status === "none" || snapshot.status === "creating") {
    return <WaitingLine>Creating the install command…</WaitingLine>;
  }
  if (snapshot.status === "failed" || snapshot.status === "revoked") {
    const failed = snapshot.status === "failed";
    return (
      <Notice
        tone={failed ? "danger" : "info"}
        title={
          failed ? "Unable to create the install command" : "Token revoked"
        }
        actions={
          <ActionButton
            label={failed ? "Try again" : "Create a new command"}
            tone="primary"
            size="sm"
            onPress={() => void store.create()}
          />
        }
      >
        {failed
          ? "Try again to get a new command."
          : "The install command adds no more machines. The machines that joined with it stay connected."}
      </Notice>
    );
  }
  const { command } = snapshot;
  if (commandExpired(command, services.env.now())) {
    return (
      <Notice
        tone="info"
        title="The install command ended"
        actions={
          <ActionButton
            label="Create a new command"
            tone="primary"
            size="sm"
            onPress={() => void store.create()}
          />
        }
      >
        A command adds machines for one hour.
      </Notice>
    );
  }
  return (
    <View style={styles.command}>
      <CodeBlock
        label="On the machine"
        lines={[command.command]}
        copy={{ text: command.command, subject: "the install command" }}
        testID="install-command"
      />
      <View style={styles.note}>
        <Text style={[text.caption, styles.noteText]}>
          The command adds any number of machines until{" "}
          {formatTime(command.expires_at)}. A machine needs{" "}
          {command.requirements}
        </Text>
        <ActionButton
          label="Revoke token"
          tone="ghost"
          size="sm"
          busy={snapshot.status === "revoking"}
          onPress={() => void store.revoke()}
        />
      </View>
      {snapshot.status === "open" && snapshot.revokeFailure ? (
        <Notice tone="danger" title="Unable to revoke the token">
          The command still works until {formatTime(command.expires_at)}. Try
          again.
        </Notice>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  command: { gap: 8 },
  note: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "flex-start",
    justifyContent: "space-between",
    gap: 8,
  },
  noteText: { flexGrow: 1, flexShrink: 1, flexBasis: 240 },
});
