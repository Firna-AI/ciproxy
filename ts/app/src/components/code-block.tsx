// A block of code in the mono face, as the mockups draw it: a head with the
// label and Copy, and the lines below. A line that starts with - or + reads
// as a diff line. The lines are DOM code elements, so a long token wraps at
// any character on a phone and stays selectable. A failed copy shows below
// the block, for the text that the block offers now.

import { StyleSheet, Text, View } from "react-native";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { CopyButton, useCopy } from "./copy.tsx";
import { Notice } from "./notice.tsx";
import { text } from "./text.ts";

export type CodeCopy = {
  /** The text that Copy writes. */
  text: string;
  /** What Copy copies, such as "the install command". */
  subject: string;
};

export function CodeBlock({
  label,
  lines,
  copy,
  testID,
}: {
  label: string;
  lines: readonly string[];
  copy?: CodeCopy;
  /** A handle for tools, such as the mask of a screenshot. */
  testID?: string;
}) {
  const copier = useCopy(copy?.text ?? "");
  return (
    <View style={styles.stack}>
      <View style={styles.block}>
        <View style={styles.head}>
          <Text style={[text.caption, styles.label]} numberOfLines={1}>
            {label}
          </Text>
          {copy ? (
            <CopyButton
              state={copier.state}
              subject={copy.subject}
              onPress={copier.copy}
            />
          ) : null}
        </View>
        <View style={styles.body} testID={testID}>
          {lines.map((line, index) => (
            <code
              key={index}
              style={{
                ...lineStyle,
                backgroundColor: line.startsWith("+")
                  ? colors.primarySoft
                  : undefined,
                color: line.startsWith("-") ? colors.muted : colors.ink,
              }}
            >
              {line}
            </code>
          ))}
        </View>
      </View>
      {copy && copier.state === "failed" ? (
        <Notice tone="danger" title={`Unable to copy ${copy.subject}`}>
          Select the text and copy it.
        </Notice>
      ) : null}
    </View>
  );
}

const styles = StyleSheet.create({
  stack: { gap: 8 },
  block: {
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.bg,
    overflow: "hidden",
  },
  head: {
    minHeight: 38,
    flexDirection: "row",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
    paddingVertical: 4,
    paddingLeft: 12,
    paddingRight: 6,
    borderBottomWidth: 1,
    borderBottomColor: colors.border,
    backgroundColor: colors.surface,
  },
  label: { flexShrink: 1 },
  body: { paddingVertical: 10, paddingHorizontal: 12, gap: 2 },
});

const lineStyle = {
  display: "block",
  fontFamily: fonts.mono,
  fontSize: 12.5,
  lineHeight: "20px",
  whiteSpace: "pre-wrap",
  wordBreak: "break-all",
} as const;
