// When jobs start on a provider: the switch that holds every provider, and
// how long a job waits for a free slot on the own machines before it goes
// to the next place of its label, with the five waits of job-routing.md.
// Each label sets the order of its places, on the Labels tab. Both controls
// save at once, and both wait while any overflow change of the workspace
// runs.

import { useId } from "react";
import { Link } from "expo-router";
import { StyleSheet, Text, View } from "react-native";
import { DropdownSelector } from "@firna/ui/dropdown";
import { Switch } from "@firna/ui/switch";

import { colors } from "@ciproxy/tokens";

import { Notice } from "../../components/notice.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import type { OverflowSettings } from "../../api/schema.ts";
import { waitOptions } from "../../view/overflow.ts";
import { settingsHref } from "../../view/settings-tabs.ts";
import type { PolicyEditor } from "./use-policy.ts";

export function WaitPanel({
  workspaceId,
  settings,
  editor,
}: {
  workspaceId: string;
  settings: OverflowSettings;
  editor: PolicyEditor;
}) {
  const labelId = useId();
  const busy = editor.locked;
  const { shown } = editor;
  return (
    <Panel title="When jobs start on a provider">
      <View style={styles.form}>
        <View style={styles.toggle}>
          <View style={styles.toggleText}>
            <Text nativeID={labelId} style={[text.body, text.strong]}>
              Use overflow providers
            </Text>
            <Text style={text.caption}>
              Each label sets the order of its places, under{" "}
              <Link
                href={settingsHref(workspaceId, "labels")}
                style={styles.link}
              >
                Labels
              </Link>
              .
            </Text>
          </View>
          <Switch
            value={shown.enabled}
            aria-labelledby={labelId}
            disabled={busy}
            onValueChange={(enabled) => void editor.save({ enabled })}
          />
        </View>
        <DropdownSelector
          variant="field"
          label="Wait for a free slot on your machines"
          options={waitOptions(settings.wait_seconds)}
          value={String(shown.wait_seconds)}
          readOnly={busy}
          onValueChange={(value) => {
            const seconds = Number(value);
            if (seconds !== shown.wait_seconds) {
              void editor.save({ wait_seconds: seconds });
            }
          }}
          hint="Then a job goes to the next place of its label."
        />
        {editor.failure ? (
          <Notice tone="danger" title={editor.failure.title}>
            {editor.failure.body}
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
  toggleText: { flexShrink: 1, gap: 2 },
  link: { color: colors.ink, textDecorationLine: "underline" },
});
