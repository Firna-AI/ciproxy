// The machine cache of one machine: its use, the cache hits of 7 days, and
// the space of the Docker disks. Clear cache deletes every entry of the
// machine cache after a confirmation. It never deletes an entry of the GitHub
// cache or a Docker disk. Only an Admin clears a cache, and only while the
// machine is online. A failed clear stays in the dialog with its reason.

import { StyleSheet, Text, View } from "react-native";

import type { ApiFailure } from "../../api/client.ts";
import type { MachineView } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import {
  DetailRow,
  FeatureDialog,
  Panel,
} from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import { cacheCopy, failureCopy } from "../../view/machines-copy.ts";
import { cacheModel } from "../../view/machines-panels.ts";
import { failureText } from "../../view/machines-outcome.ts";
import { Meter } from "./machine-figures.tsx";

export function CachePanel({
  machine,
  onClear,
}: {
  machine: MachineView;
  /** Opens the confirmation; null when the person cannot clear the cache. */
  onClear: (() => void) | null;
}) {
  const model = cacheModel(machine);
  return (
    <Panel
      title={cacheCopy.title}
      action={
        onClear === null ? undefined : (
          <ActionButton label={cacheCopy.clear} size="sm" onPress={onClear} />
        )
      }
    >
      {model.reported ? (
        <>
          <Meter
            label={cacheCopy.disk}
            ratio={model.ratio}
            value={model.value}
          />
          <View>
            <DetailRow label={cacheCopy.hits} first>
              <Text style={text.small}>{model.hits}</Text>
            </DetailRow>
            <DetailRow label={cacheCopy.docker}>
              <Text style={text.small}>{model.docker}</Text>
            </DetailRow>
          </View>
        </>
      ) : (
        <Text style={text.small}>{model.text}</Text>
      )}
    </Panel>
  );
}

export function ClearCacheDialog({
  machine,
  visible,
  busy,
  failure,
  onCancel,
  onConfirm,
}: {
  machine: MachineView;
  visible: boolean;
  busy: boolean;
  failure: ApiFailure | null;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const model = cacheModel(machine);
  return (
    <FeatureDialog
      title={cacheCopy.dialogTitle(machine.hostname)}
      visible={visible}
      busy={busy}
      onClose={onCancel}
      footer={
        <View style={styles.footer}>
          <ActionButton
            label={cacheCopy.cancel}
            tone="ghost"
            disabled={busy}
            onPress={onCancel}
          />
          <ActionButton
            label={cacheCopy.clear}
            tone="danger"
            busy={busy}
            onPress={onConfirm}
          />
        </View>
      }
    >
      <View style={styles.body}>
        <Text style={text.small}>
          {cacheCopy.dialogBody(model.reported ? model.used : null)}
        </Text>
        {failure ? (
          <Notice
            tone="danger"
            title={failureCopy["clear-cache"](machine.hostname)}
          >
            {failureText(failure)}
          </Notice>
        ) : null}
      </View>
    </FeatureDialog>
  );
}

const styles = StyleSheet.create({
  footer: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "flex-end",
    gap: 8,
  },
  body: { gap: 12 },
});
