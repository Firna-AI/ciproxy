// The one-line install command of the machine step. The store creates one
// command when the step first shows, never on a refresh. The command shows
// with Copy and Revoke token, its end, and what a machine needs. After
// Revoke token or the end, a new command is one press away.

import { useEffect, useState } from "react";
import { StyleSheet, Text, View } from "react-native";

import { ActionButton } from "../../components/actions.tsx";
import { CodeBlock } from "../../components/code-block.tsx";
import { CopyIcon } from "../../components/icons.tsx";
import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { useServices } from "../../services/context.tsx";
import { useValue } from "../../services/hooks.ts";
import { commandExpired } from "../../state/install-command.ts";
import { formatTime } from "../../view/format.ts";

type CopyState = "idle" | "copied" | "failed";

async function copy(value: string): Promise<boolean> {
  try {
    await navigator.clipboard.writeText(value);
    return true;
  } catch {
    return false;
  }
}

export function InstallCommandView({ workspaceId }: { workspaceId: string }) {
  const services = useServices();
  const store = services.installCommand(workspaceId);
  const snapshot = useValue(store.state);
  const [copied, setCopied] = useState<CopyState>("idle");
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
        testID="install-command"
      />
      <View style={styles.row}>
        <ActionButton
          label={copied === "copied" ? "Copied" : "Copy"}
          icon={CopyIcon}
          size="sm"
          onPress={() =>
            void copy(command.command).then((done) =>
              setCopied(done ? "copied" : "failed"),
            )
          }
        />
        <ActionButton
          label="Revoke token"
          tone="ghost"
          size="sm"
          busy={snapshot.status === "revoking"}
          onPress={() => void store.revoke()}
        />
      </View>
      {copied === "failed" ? (
        <Notice tone="warning" title="Unable to copy the command">
          Select the command and copy it.
        </Notice>
      ) : null}
      {snapshot.status === "open" && snapshot.revokeFailure ? (
        <Notice tone="danger" title="Unable to revoke the token">
          The command still works until {formatTime(command.expires_at)}. Try
          again.
        </Notice>
      ) : null}
      <Text style={text.caption}>
        The command adds any number of machines until{" "}
        {formatTime(command.expires_at)}. A machine needs {command.requirements}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  command: { gap: 8 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
});
