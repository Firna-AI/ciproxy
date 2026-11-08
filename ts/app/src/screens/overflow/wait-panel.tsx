// When jobs start on Boat: the overflow switch and the wait before a job
// starts on Boat, with the five waits of job-routing.md. Both save at once,
// one save at a time.

import { useId } from "react";
import { StyleSheet, Text, View } from "react-native";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Switch } from "@firna/ui/switch";

import { Notice } from "../../components/notice.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import type { OverflowSettings } from "../../api/schema.ts";
import { waitOptions } from "../../view/boat.ts";
import type { PolicyEditor } from "./use-policy.ts";

export function WaitPanel({
  settings,
  editor,
}: {
  settings: OverflowSettings;
  editor: PolicyEditor;
}) {
  const labelId = useId();
  const busy = editor.saving !== null;
  const { shown } = editor;
  return (
    <Panel title="When jobs start on Boat">
      <View style={styles.form}>
        <View style={styles.toggle}>
          <Text
            nativeID={labelId}
            style={[text.body, text.strong, styles.label]}
          >
            Use Boat when your machines are busy
          </Text>
          <Switch
            value={shown.enabled}
            aria-labelledby={labelId}
            disabled={busy}
            onValueChange={(enabled) => void editor.save("wait", { enabled })}
          />
        </View>
        <DropdownSelector
          variant="field"
          label="Start on Boat after a job waits"
          options={waitOptions(settings.wait_seconds)}
          value={String(shown.wait_seconds)}
          readOnly={busy}
          onValueChange={(value) => {
            const seconds = Number(value);
            if (seconds !== shown.wait_seconds) {
              void editor.save("wait", { wait_seconds: seconds });
            }
          }}
          hint="Warm runners on your machines take jobs first."
        />
        {editor.waitFailure ? (
          <Notice tone="danger" title={editor.waitFailure.title}>
            {editor.waitFailure.body}
          </Notice>
        ) : null}
      </View>
    </Panel>
  );
}

const styles = StyleSheet.create({
  form: { gap: 16, maxWidth: 560 },
  toggle: {
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 16,
  },
  label: { flexShrink: 1 },
});
