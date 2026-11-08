// The fit line of the label dialog. It shows the fit of the size in the
// fields once the API answers for that size; until then it says that it
// checks, so the line never shows the fit of an older size.

import { Text } from "react-native";

import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import { text } from "../../components/text.ts";
import type { PreviewSnapshot } from "../../state/settings-preview.ts";
import { fitKey, type DraftCheck } from "../../view/settings-label-form.ts";
import { fitLine } from "../../view/settings-labels.ts";

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
  const line = fitLine(check.input, shown.fit);
  if (line.kind === "warning") {
    return (
      <Notice tone="warning" title={line.title}>
        {line.body}
      </Notice>
    );
  }
  return (
    <Text role="status" style={text.caption}>
      {line.text}
    </Text>
  );
}
