// Disconnect asks first: ciproxy then starts no new jobs on Boat, the jobs
// that run on Boat finish, and ciproxy deletes the key when the last of their
// sandboxes ends. The dialog opens on Cancel, the safe action, and a phone
// shows it as a sheet. The dialog opens only after a press, never in the
// static export, so it reads the window width at once. Firna builds the
// sheet and the centred dialog as different trees and moves focus only when
// it opens, so a change of placement opens a new frame, which focuses Cancel
// again.

import { useRef, useState } from "react";
import { StyleSheet, Text, useWindowDimensions, View } from "react-native";
import { WebModalFrame } from "@firna/ui/modal";

import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import { viewportFor } from "../../components/viewport.ts";
import { useValue } from "../../services/hooks.ts";
import type { OverflowStore } from "../../state/overflow.ts";
import { disconnectFailure } from "../../view/boat-failures.ts";
import type { FailureCopy } from "../../view/claim.ts";

export function DisconnectDialog({
  store,
  onClose,
  onDisconnected,
}: {
  store: OverflowStore;
  onClose: () => void;
  onDisconnected: () => void;
}) {
  const placement =
    viewportFor(useWindowDimensions().width) === "mobile"
      ? "bottom-sheet"
      : "center";
  const [busy, setBusy] = useState(false);
  const locked = useValue(store.pending) && !busy;
  const [failure, setFailure] = useState<FailureCopy | null>(null);
  const running = useRef(false);
  const cancel = useRef<View>(null);
  const disconnect = async () => {
    if (running.current) return;
    running.current = true;
    setBusy(true);
    setFailure(null);
    const result = await store.disconnect();
    running.current = false;
    setBusy(false);
    if (result.ok) onDisconnected();
    else setFailure(disconnectFailure(result.failure));
  };
  return (
    <WebModalFrame
      key={placement}
      title="Disconnect Boat?"
      size="sm"
      placement={placement}
      dismissible={!busy}
      closeDisabled={busy}
      initialFocusRef={cancel}
      onClose={onClose}
      footer={
        <View style={styles.footer}>
          <ActionButton
            label="Cancel"
            tone="ghost"
            disabled={busy}
            buttonRef={cancel}
            onPress={onClose}
          />
          <ActionButton
            label="Disconnect"
            tone="danger"
            busy={busy}
            disabled={locked}
            onPress={() => void disconnect()}
          />
        </View>
      }
    >
      <View style={styles.body}>
        <Text style={text.small}>
          ciproxy starts no new jobs on Boat, and jobs run on your machines
          only.
        </Text>
        <Text style={text.small}>
          Jobs that run on Boat now finish. ciproxy then deletes the key.
        </Text>
        {failure ? (
          <Notice tone="danger" title={failure.title}>
            {failure.body}
          </Notice>
        ) : null}
      </View>
    </WebModalFrame>
  );
}

const styles = StyleSheet.create({
  body: { gap: 8 },
  footer: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "flex-end",
    gap: 8,
  },
});
