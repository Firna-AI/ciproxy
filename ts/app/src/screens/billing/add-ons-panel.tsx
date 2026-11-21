// The add-ons of Plan and usage. Free names the plans that have add-ons. On a
// paid plan, a payer sets the quantities and saves a choice whose monthly
// total the API priced: the saved add-ons, or those that cover the use. A
// person who cannot pay reads the saved quantities and their total.

import { StyleSheet, Text, View } from "react-native";

import { colors } from "@ciproxy/tokens";

import type { PlanAddOns, PlanUsagePage } from "../../api/billing-types.ts";
import { ActionButton } from "../../components/actions.tsx";
import { Quantity } from "../../components/billing/quantity.tsx";
import { DetailList, DetailRow } from "../../components/detail-list.tsx";
import { Panel } from "../../components/panel.tsx";
import { text } from "../../components/text.ts";
import {
  addOnQuote,
  quoteText,
  suggestedAddOns,
} from "../../view/billing-addons.ts";
import {
  formatAddOn,
  monthlyText,
  tierNames,
} from "../../view/billing-format.ts";
import type { OutcomeModel } from "../../view/billing-outcome.ts";
import { PaymentResult } from "./payment-result.tsx";

const captions = {
  starts: "Blocks of starts for each month",
  machines: "Own machines above the plan",
} as const;

function SavedAddOns({ page }: { page: PlanUsagePage }) {
  const { add_ons: addOns } = page.status.selection;
  return (
    <Panel title="Add-ons">
      <DetailList>
        <DetailRow label="Extra starts">
          {formatAddOn("starts", addOns.start_blocks)}
        </DetailRow>
        <DetailRow label="Extra own machines">
          {formatAddOn("machines", addOns.machines)}
        </DetailRow>
        <DetailRow label="Your plan with add-ons">
          {monthlyText(page.current.monthly_cents)}
        </DetailRow>
      </DetailList>
    </Panel>
  );
}

export function AddOnsPanel({
  page,
  edited,
  onEdit,
  canSave,
  saving,
  outcome,
  onSave,
  onRetry,
  onReload,
  onPortal,
}: {
  page: PlanUsagePage;
  edited: PlanAddOns;
  onEdit: (addOns: PlanAddOns) => void;
  /** No payment or operation waits, and the latest read succeeded. */
  canSave: boolean;
  saving: boolean;
  outcome: OutcomeModel | null;
  onSave: () => void;
  onRetry: () => void;
  /** Read Plan and usage again, after that read failed. */
  onReload: () => void;
  onPortal: () => void;
}) {
  const { tier } = page.status.selection;
  if (tier === "free") {
    return (
      <Panel title="Add-ons">
        <Text style={text.small}>
          {`${tierNames.team} and ${tierNames.business} can add starts and own machines.`}
        </Text>
      </Panel>
    );
  }
  if (!page.status.can_pay) return <SavedAddOns page={page} />;
  const quote = addOnQuote(page, edited);
  const words = quoteText(quote);
  const suggested = suggestedAddOns(page);
  const offerShown = quote.kind === "offer";
  return (
    <Panel title="Add-ons">
      <View style={styles.stack}>
        <Quantity
          label="Extra starts"
          caption={captions.starts}
          value={edited.start_blocks}
          disabled={saving}
          onChange={(value) => onEdit({ ...edited, start_blocks: value })}
        />
        <Quantity
          label="Extra own machines"
          caption={captions.machines}
          value={edited.machines}
          disabled={saving}
          onChange={(value) => onEdit({ ...edited, machines: value })}
        />
        <View style={styles.total}>
          <View style={styles.totalText}>
            <Text style={[text.body, text.strong]}>{words.total}</Text>
            <Text style={text.caption}>{words.detail}</Text>
          </View>
          <View style={styles.buttons}>
            {suggested !== null && !offerShown ? (
              <ActionButton
                label="Use the add-ons that cover your use"
                tone="ghost"
                disabled={saving}
                onPress={() => onEdit(suggested)}
              />
            ) : null}
            <ActionButton
              label="Save add-ons"
              tone={offerShown ? "primary" : "secondary"}
              busy={saving}
              disabled={!offerShown || !canSave}
              onPress={onSave}
            />
          </View>
        </View>
        {outcome ? (
          <PaymentResult
            model={outcome}
            busy={saving}
            onRetry={onRetry}
            onReload={onReload}
            onPortal={onPortal}
          />
        ) : null}
      </View>
    </Panel>
  );
}

const styles = StyleSheet.create({
  stack: { gap: 16 },
  total: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 12,
    paddingTop: 12,
    borderTopWidth: 1,
    borderTopColor: colors.border,
  },
  totalText: { flexGrow: 1, flexShrink: 1, flexBasis: 220, gap: 2 },
  // Never wider than the panel, so the two buttons wrap on a phone.
  buttons: { flexDirection: "row", flexWrap: "wrap", gap: 8, maxWidth: "100%" },
});
