// A page in the signed-out frame, rendered from its model: the title, the
// lead, one notice, one list, the actions, and the footer.

import type { ReactNode } from "react";
import { StyleSheet, Text, View } from "react-native";

import { colors, radii } from "@ciproxy/tokens";

import type { AuthPageModel } from "../view/page-model.ts";
import { AuthShell } from "./auth-shell.tsx";
import { Notice } from "./notice.tsx";
import { PageActionView, type ActionHandlers } from "./page-actions.tsx";
import { text } from "./text.ts";

export function AuthPage({
  model,
  handlers = {},
  failure,
  footer,
  children,
}: {
  model: AuthPageModel;
  handlers?: ActionHandlers;
  /** A failed action of this page, with what to do next. */
  failure?: { title: string; body: string } | null;
  /** A footer with links, in place of the plain footer of the model. */
  footer?: ReactNode;
  children?: ReactNode;
}) {
  return (
    <AuthShell
      footer={footer ?? (model.footer === "" ? undefined : model.footer)}
    >
      <View style={styles.heading}>
        <Text role="heading" aria-level={1} style={text.title}>
          {model.title}
        </Text>
        {model.lead ? <Text style={text.lead}>{model.lead}</Text> : null}
      </View>
      {model.notice ? (
        <Notice tone={model.notice.tone} title={model.notice.title}>
          {model.notice.body}
        </Notice>
      ) : null}
      {failure ? (
        <Notice tone="danger" title={failure.title}>
          {failure.body}
        </Notice>
      ) : null}
      {model.list ? (
        <View style={styles.listBlock}>
          <Text style={text.caption}>{model.list.caption}</Text>
          <View style={styles.list} role="list">
            {model.list.items.map((item) => (
              <Text key={item} role="listitem" style={text.small}>
                {item}
              </Text>
            ))}
          </View>
        </View>
      ) : null}
      {children}
      {model.actions.length > 0 ? (
        <View style={styles.actions}>
          {model.actions.map((action) => (
            <PageActionView
              key={action.label}
              action={action}
              handlers={handlers}
            />
          ))}
        </View>
      ) : null}
    </AuthShell>
  );
}

const styles = StyleSheet.create({
  heading: { gap: 6 },
  listBlock: { gap: 6 },
  list: {
    gap: 10,
    paddingVertical: 12,
    paddingHorizontal: 16,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.lg,
    backgroundColor: colors.bg,
  },
  actions: { gap: 10 },
});
