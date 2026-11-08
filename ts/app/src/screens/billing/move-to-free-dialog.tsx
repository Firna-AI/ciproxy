// The confirmation of Move to Free. The paid plan stays until the end of its
// period, then the workspace is on Free, and jobs keep running on every own
// machine. The dialog is a sheet on a phone. Keep closes it; Move to Free
// asks the API, then Plan and usage shows the day on which the plan ends.

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
            label={`Keep ${current}`}
            tone="ghost"
            disabled={working}
            onPress={onClose}
          />
          <ActionButton
            label="Move to Free"
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
        {state.status === "failed" ? (
          <Notice tone="danger" title="Move to Free did not finish">
            Your plan stays as it was. Try again.
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
