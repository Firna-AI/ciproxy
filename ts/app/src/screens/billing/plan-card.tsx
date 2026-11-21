// One plan on the Plans page: its price, what it includes, its action, and
// what it adds to the plan before it. "Coming later" marks a feature that
// ships after the launch. A full ink border marks the plan that covers the
// use, and a person who cannot pay reads who can in place of the action.

import { StyleSheet, Text, View } from "react-native";
import { Link } from "expo-router";
import { Badge } from "@firna/ui/badge";

import { colors, fonts, radii } from "@ciproxy/tokens";

import { ActionButton, ActionLink } from "../../components/actions.tsx";
import { CheckIcon, InfoIcon } from "../../components/icons.tsx";
import { RoleNote } from "../../components/role-note.tsx";
import { text } from "../../components/text.ts";
import { askPayer } from "../../view/billing-format.ts";
import type { PlanCardModel } from "../../view/billing-plans.ts";

function CardAction({
  model,
  onMoveToFree,
}: {
  model: PlanCardModel;
  onMoveToFree: () => void;
}) {
  const { action } = model;
  if (action.kind === "current") {
    return (
      <ActionButton
        label="Current plan"
        block
        disabled
        onPress={() => undefined}
      />
    );
  }
  if (action.kind === "ask") return <RoleNote>{askPayer}</RoleNote>;
  const tone = action.primary ? "primary" : "secondary";
  if (action.kind === "move-to-free" || !action.enabled) {
    return (
      <ActionButton
        label={action.label}
        tone={tone}
        block
        disabled={!action.enabled}
        onPress={onMoveToFree}
      />
    );
  }
  return (
    <Link href={action.href} asChild>
      <ActionLink label={action.label} href={action.href} tone={tone} block />
    </Link>
  );
}

export function PlanCard({
  model,
  onMoveToFree,
}: {
  model: PlanCardModel;
  onMoveToFree: () => void;
}) {
  return (
    <View
      style={[styles.card, model.tag === "covers" && styles.covers]}
      role="region"
      aria-label={model.name}
    >
      <View style={styles.head}>
        <Text role="heading" aria-level={2} style={text.heading}>
          {model.name}
        </Text>
        {model.tag === "current" ? (
          <Badge tone="neutral" variant="outline" size="sm">
            Current plan
          </Badge>
        ) : model.tag === "covers" ? (
          <Badge tone="primary" variant="solid" size="sm">
            Covers your use
          </Badge>
        ) : null}
      </View>
      <Text style={styles.price}>
        <Text style={styles.amount}>{model.price}</Text>
        <Text style={text.lead}> a month</Text>
      </Text>
      <Text style={text.small}>{model.capacity}</Text>
      <CardAction model={model} onMoveToFree={onMoveToFree} />
      <View style={styles.features}>
        <Text style={text.caption}>{model.lead}</Text>
        {model.features.map((feature) => (
          <View key={feature.name} style={styles.feature}>
            {feature.comingLater ? (
              <InfoIcon size={14} color={colors.muted} />
            ) : (
              <CheckIcon size={14} color={colors.ink} />
            )}
            <View style={styles.featureText}>
              <Text style={text.small}>{feature.name}</Text>
              {feature.comingLater ? (
                <Text style={text.caption}>Coming later</Text>
              ) : null}
            </View>
          </View>
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  card: {
    flex: 1,
    minWidth: 0,
    gap: 14,
    padding: 20,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.xl,
    backgroundColor: colors.surface,
  },
  covers: { borderColor: colors.ink, borderWidth: 2, padding: 19 },
  head: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    justifyContent: "space-between",
    gap: 8,
  },
  price: { fontFamily: fonts.sans, color: colors.ink },
  amount: {
    fontFamily: fonts.sans,
    fontSize: 30,
    fontWeight: "600",
    letterSpacing: -0.9,
    lineHeight: 36,
    color: colors.ink,
  },
  features: { gap: 8 },
  feature: { flexDirection: "row", alignItems: "flex-start", gap: 8 },
  featureText: { flexShrink: 1, gap: 2 },
});
