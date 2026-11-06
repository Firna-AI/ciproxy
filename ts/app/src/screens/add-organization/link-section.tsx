// One section of the return page: one link of the person, or the requests
// that wait for an owner. The choice of a workspace lists the offers of the
// API with none picked, and the button names the organization and then the
// workspace.

import { StyleSheet, Text, View } from "react-native";
import { Button } from "@firna/ui/button";
import { RadioCard, RadioCardGroup } from "@firna/ui/radio";

import { Notice } from "../../components/notice.tsx";
import { WaitingLine } from "../../components/page-state.tsx";
import {
  PageActionView,
  type ActionHandlers,
} from "../../components/page-actions.tsx";
import { text } from "../../components/text.ts";
import type { LinkSection } from "../../view/add-organization.ts";
import type { NoticeModel } from "../../view/page-model.ts";

export function LinkSectionView({
  section,
  failure,
  handlers,
  onPick,
  onSubmit,
}: {
  section: LinkSection;
  failure: NoticeModel | null;
  handlers: ActionHandlers;
  onPick: (workspaceId: string) => void;
  onSubmit: () => void;
}) {
  return (
    <View style={styles.section}>
      <View style={styles.heading}>
        <Text role="heading" aria-level={1} style={text.title}>
          {section.title}
        </Text>
        {section.lead ? <Text style={text.lead}>{section.lead}</Text> : null}
      </View>
      {section.notice ? (
        <Notice tone={section.notice.tone} title={section.notice.title}>
          {section.notice.body}
        </Notice>
      ) : null}
      {failure ? (
        <Notice tone={failure.tone} title={failure.title}>
          {failure.body}
        </Notice>
      ) : null}
      {section.choices ? (
        <RadioCardGroup label="Workspaces where you are an Admin">
          {section.choices.map((choice) => (
            <RadioCard
              key={choice.id}
              title={choice.name}
              body={choice.note ?? undefined}
              checked={choice.checked}
              onPress={() => onPick(choice.id)}
            />
          ))}
        </RadioCardGroup>
      ) : null}
      {section.waiting ? <WaitingLine>{section.waiting}</WaitingLine> : null}
      {section.reasons.length > 0 ? (
        <View style={styles.reasons} role="list">
          {section.reasons.map((reason) => (
            <Text key={reason} role="listitem" style={text.small}>
              {reason}
            </Text>
          ))}
        </View>
      ) : null}
      <View style={styles.actions}>
        {section.submit ? (
          <Button
            tone="primary"
            size="lg"
            block
            busy={section.submit.busy}
            disabled={!section.submit.enabled}
            onPress={section.submit.enabled ? onSubmit : undefined}
          >
            {section.submit.label}
          </Button>
        ) : null}
        {section.actions.map((action) => (
          <PageActionView
            key={action.label}
            action={action}
            handlers={handlers}
          />
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  section: { gap: 20 },
  heading: { gap: 6 },
  reasons: { gap: 6 },
  actions: { gap: 10 },
});
