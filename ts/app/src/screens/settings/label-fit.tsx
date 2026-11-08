// The fit of the label dialog: the VM disk of a job of the size, and the line
// that says where jobs of the size run. Both show once the API answers for
// the size and the switches in the fields; until then the dialog says that
// it checks, so it never shows the fit of an older draft.

import { StyleSheet, Text, View } from "react-native";

import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import { fitKey, type PreviewSnapshot } from "../../state/settings-preview.ts";
import type { DraftCheck } from "../../view/settings-label-form.ts";
import { fitLine } from "../../view/settings-label-fit.ts";
import { vmDiskLine } from "../../view/settings-label-sizes.ts";

export function LabelFitLine({
  check,
  preview,
}: {
  check: DraftCheck;
  preview: PreviewSnapshot;
}) {
  if (!check.ok) return null;
  const { shown } = preview;
  if (shown === null || fitKey(shown.input) !== fitKey(check.input)) {
    if (preview.failure !== null && !preview.pending) {
      return (
        <Text role="status" style={text.caption}>
          Unable to check how many jobs of this size fit. Change a value to
          check again.
        </Text>
      );
    }
    return <WaitingLine>Checking how many jobs of this size fit…</WaitingLine>;
  }
  const disk = vmDiskLine(shown.fit);
  const line = fitLine(check.input, shown.fit);
  return (
    <View style={styles.fit}>
      {disk === null ? null : <Text style={text.caption}>{disk}</Text>}
      {line.kind === "warning" ? (
        <Notice tone="warning" title={line.title}>
          {line.body}
        </Notice>
      ) : (
        <Text role="status" style={text.caption}>
          {line.text}
        </Text>
      )}
    </View>
  );
}

const styles = StyleSheet.create({ fit: { gap: 12 } });
