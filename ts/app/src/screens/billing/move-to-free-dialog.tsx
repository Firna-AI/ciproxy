// The confirmation of Move to Free. The paid plan stays until the end of its
// period, then the workspace is on Free, and jobs keep running on every own
// machine. The dialog is a sheet on a phone. Keep closes it; Move to Free
// asks the API, then Plan and usage shows the day on which the plan ends.
// When the page did not get the result, the dialog says so and points to
// Plan and usage. It never says then that the plan stayed as it was. Its
// dismiss button then says Close, not Keep, because Move to Free may have
// taken effect.

import { StyleSheet, Text, View } from "react-native";
import { WebModalFrame } from "@firna/ui/modal";

import type { PlansPage } from "../../api/billing-types.ts";
import { ActionButton } from "../../components/actions.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import { useViewport } from "../../components/viewport.ts";
import type { ActionSnapshot } from "../../state/billing-actions.ts";
import {
  capacityText,
  dayText,
  lastDayText,
  machinesText,
  tierNames,
} from "../../view/billing-format.ts";
import { moveToFreeFailure } from "../../view/billing-uncertain.ts";

export function MoveToFreeDialog({
  page,
  workspace,
  state,
  onConfirm,
  onClose,
}: {
  page: PlansPage;
  workspace: string;
  state: ActionSnapshot;
  onConfirm: () => void;
  onClose: () => void;
}) {
  const mobile = useViewport() === "mobile";
  const { usage } = page;
  const current = tierNames[usage.status.selection.tier];
  const free = page.plans.find((card) => card.plan.selection.tier === "free");
  const working = state.status === "working";
  const failure =
    state.status === "failed"
      ? moveToFreeFailure(
          state.failure,
          current,
          lastDayText(usage.status.period_end),
        )
      : null;
  return (
    <WebModalFrame
      visible
      title={`Move ${workspace} to Free?`}
      onClose={onClose}
      closeDisabled={working}
      placement={mobile ? "bottom-sheet" : "center"}
      size="sm"
      footer={
        <View style={styles.footer}>
          <ActionButton
            label={failure?.dismissLabel ?? `Keep ${current}`}
            tone="ghost"
            disabled={working}
            onPress={onClose}
          />
          <ActionButton
            label={failure?.retryLabel ?? "Move to Free"}
            tone="primary"
            busy={working}
            onPress={onConfirm}
          />
        </View>
      }
    >
      <View style={styles.body}>
        <Text style={text.small}>
          {`${current} stays until ${lastDayText(usage.status.period_end)}, the end of this period. From ${dayText(usage.status.period_end)}, ${workspace} is on Free${free ? `: ${capacityText(free.plan)}` : ""}.`}
        </Text>
        <Text style={text.small}>
          {`Jobs keep running on all ${machinesText(usage.machines)}.`}
        </Text>
        {failure ? (
          <Notice tone={failure.tone} title={failure.title}>
            {failure.body}
          </Notice>
        ) : null}
      </View>
    </WebModalFrame>
  );
}

const styles = StyleSheet.create({
  body: { gap: 10 },
  footer: {
    flexDirection: "row",
    flexWrap: "wrap",
    justifyContent: "flex-end",
    gap: 8,
  },
});
