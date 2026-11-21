// The machine cache of one machine: its use, the cache hits of 7 days, and
// the space of the Docker disks, from the last report. A machine with no
// report shows no figures. Otherwise the machine cache and the Docker disks
// each show their figures, or Unknown when the machine did not measure them,
// with one note that names the parts that show Unknown. Clear cache deletes
// every entry of the machine cache after a confirmation. It never deletes an
// entry of the GitHub cache or a Docker disk. Only an Admin clears a cache,
// and only while the machine is online. A failed clear stays in the dialog
// with its reason.

import { StyleSheet, Text, View } from "react-native";

import type { ApiFailure } from "../../api/client.ts";
import type { CacheView, MachineView } from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import {
  DetailRow,
  FeatureDialog,
  Panel,
} from "../../components/feature-parts.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import {
  cacheRatio,
  cacheUse,
  clearCacheBody,
  dockerUse,
  machineHits,
  unknownNote,
} from "../../view/machines-cache.ts";
import { cacheCopy, failureCopy } from "../../view/machines-copy.ts";
import { failureText } from "../../view/machines-outcome.ts";
import { Meter, UnknownMeter, UnknownValue } from "./machine-figures.tsx";

/** The figures of a stored report: each part measured, or Unknown. */
function CacheFigures({ cache }: { cache: CacheView }) {
  const figures = cache.machine_cache;
  const docker = cache.docker_disks;
  const note = unknownNote(cache);
  return (
    <>
      {figures.status === "measured" ? (
        <Meter
          label={cacheCopy.disk}
          ratio={cacheRatio(figures)}
          value={cacheUse(figures)}
        />
      ) : (
        <UnknownMeter label={cacheCopy.disk} />
      )}
      <View>
        <DetailRow label={cacheCopy.hits} first>
          {figures.status === "measured" ? (
            <Text style={text.small}>{machineHits(figures)}</Text>
          ) : (
            <UnknownValue />
          )}
        </DetailRow>
        <DetailRow label={cacheCopy.docker}>
          {docker.status === "measured" ? (
            <Text style={text.small}>{dockerUse(docker)}</Text>
          ) : (
            <UnknownValue />
          )}
        </DetailRow>
      </View>
      {note === null ? null : (
        <Text style={[text.caption, styles.note]}>{note}</Text>
      )}
    </>
  );
}

export function CachePanel({
  machine,
  onClear,
}: {
  machine: MachineView;
  /** Opens the confirmation; null when the person cannot clear the cache. */
  onClear: (() => void) | null;
}) {
  const cache = machine.cache ?? null;
  return (
    <Panel
      title={cacheCopy.title}
      action={
        onClear === null ? undefined : (
          <ActionButton label={cacheCopy.clear} size="sm" onPress={onClear} />
        )
      }
    >
      {cache === null ? (
        <Text style={text.small}>{cacheCopy.unreported}</Text>
      ) : (
        <CacheFigures cache={cache} />
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
        <Text style={text.small}>{clearCacheBody(machine)}</Text>
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
  note: { marginTop: -2 },
});
