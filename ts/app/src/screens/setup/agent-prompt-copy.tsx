// Copy prompt for your agent: the button of the first-job step that copies
// the prompt for a coding agent. The prompt stays out of the page while the
// copy works. When the copy fails, the page shows the prompt, so that the
// person can select it and copy it.

import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import { CopyButton, useCopy } from "../../components/copy.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";

const subject = "the prompt for your agent";

export function AgentPromptCopy({ prompt }: { prompt: string }) {
  const copier = useCopy(prompt);
  return (
    <View style={styles.stack}>
      <View style={styles.row}>
        <CopyButton
          state={copier.state}
          subject={subject}
          label="Copy prompt for your agent"
          onPress={copier.copy}
        />
      </View>
      {copier.state === "failed" ? (
        <>
          <Notice tone="danger" title={`Unable to copy ${subject}`}>
            Select the prompt below and copy it.
          </Notice>
          <View style={styles.prompt}>
            <Text style={text.small} selectable>
              {prompt}
            </Text>
          </View>
        </>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  stack: { gap: 8 },
  row: { flexDirection: "row", flexWrap: "wrap", gap: 8 },
  prompt: {
    paddingVertical: 10,
    paddingHorizontal: 12,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.bg,
  },
});
