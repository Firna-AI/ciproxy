// The order of a label in the label dialog: the places where its jobs run,
// in order. A grab handle moves a place, by pointer or by keyboard, when the
// order holds more than one, and Remove takes a place out. Add a place offers
// own machines and each offered provider that the order does not hold. Each
// row says what the place does with a job of the label's size. An empty
// order holds the jobs. A macOS label holds own machines only, with no
// handle, no Remove, and no Add. The list is the sortable list of Firna.

import { StyleSheet, Text, View } from "react-native";
import { SortableList, applySortableMove } from "@firna/ui/sortable-list";

import { colors, fonts, radii } from "@ciproxy/tokens";

import type {
  LabelFit,
  LabelInput,
  LabelPlace,
  OverflowProvider,
} from "../../api/schema.ts";
import { ActionButton } from "../../components/actions.tsx";
import { PlusIcon } from "../../components/icons.tsx";
import { InlineLink } from "../../components/inline-link.tsx";
import { Notice } from "../../components/notice.tsx";
import { text } from "../../components/text.ts";
import { placeCaption } from "../../view/settings-label-order.ts";
import {
  addablePlaces,
  placeKey,
  placeName,
  samePlace,
} from "../../view/settings-label-places.ts";

/** The facts that the rows read: the draft, its fit, and the overflow wait. */
export type OrderFacts = {
  /** The label of the draft, or null while its size needs a fix. */
  label: LabelInput | null;
  /** The fit that holds for the draft, or null while it loads. */
  fit: LabelFit | null;
  waitSeconds: number | null;
};

function OrderRow({
  place,
  index,
  facts,
  mac,
  onRemove,
}: {
  place: LabelPlace;
  index: number;
  facts: OrderFacts;
  mac: boolean;
  onRemove: (() => void) | null;
}) {
  const platform = mac ? "macos_arm64" : "linux_x64";
  const caption =
    facts.label === null
      ? null
      : placeCaption(facts.label, place, index, facts.fit, facts.waitSeconds);
  return (
    <View style={styles.row}>
      <View style={styles.index} aria-hidden>
        <Text style={styles.indexText}>{index + 1}</Text>
      </View>
      <View style={styles.rowText}>
        <Text style={[text.small, text.strong]}>
          {placeName(place, platform)}
        </Text>
        {caption === null ? null : <Text style={text.caption}>{caption}</Text>}
      </View>
      {onRemove === null ? null : (
        <ActionButton
          label="Remove"
          accessibilityLabel={`Remove ${placeName(place, platform)}`}
          size="sm"
          tone="ghost"
          onPress={onRemove}
        />
      )}
    </View>
  );
}

function AddPlace({
  order,
  offered,
  overflowHref,
  onAdd,
}: {
  order: readonly LabelPlace[];
  /** The offered providers, or null while the overflow settings are unknown. */
  offered: readonly OverflowProvider[] | null;
  overflowHref: string;
  onAdd: (place: LabelPlace) => void;
}) {
  const places = addablePlaces(order, offered ?? []);
  return (
    <>
      {places.length > 0 ? (
        <View style={styles.add}>
          <Text style={text.caption}>Add a place</Text>
          {places.map((place) => (
            <ActionButton
              key={placeKey(place)}
              label={placeName(place, "linux_x64")}
              accessibilityLabel={`Add ${placeName(place, "linux_x64")}`}
              size="sm"
              icon={PlusIcon}
              onPress={() => onAdd(place)}
            />
          ))}
        </View>
      ) : null}
      {offered !== null && offered.length === 0 ? (
        <Text style={text.caption}>
          To add a provider, connect it in{" "}
          <InlineLink
            label="Settings under Overflow"
            href={overflowHref}
            newTab
          />
          .
        </Text>
      ) : null}
    </>
  );
}

export function LabelOrder({
  order,
  mac,
  facts,
  offered,
  overflowHref,
  name,
  onChange,
}: {
  order: readonly LabelPlace[];
  /** A macOS label, which runs on the Macs only. */
  mac: boolean;
  facts: OrderFacts;
  offered: readonly OverflowProvider[] | null;
  overflowHref: string;
  /** The name of the label, for the name of the list. */
  name: string;
  onChange: (order: LabelPlace[]) => void;
}) {
  const movable = !mac && order.length > 1;
  return (
    <View style={styles.order}>
      <Text style={[text.small, text.strong]}>Where jobs run</Text>
      <Text style={text.caption}>
        {mac
          ? "A macOS label runs on your Macs only."
          : "A job goes to the next place when a place cannot take it."}
      </Text>
      {order.length === 0 ? (
        <Notice tone="warning" title="Jobs with this label wait">
          The order has no place. Add a place to run them.
        </Notice>
      ) : (
        <SortableList
          accessibilityLabel={
            name === "" ? "Where jobs run" : `Where jobs of ${name} run`
          }
          items={[...order]}
          itemKey={placeKey}
          itemLabel={(place) =>
            placeName(place, mac ? "macos_arm64" : "linux_x64")
          }
          handle={movable ? "start" : undefined}
          onReorder={
            movable
              ? (move) =>
                  onChange(applySortableMove([...order], move, placeKey))
              : undefined
          }
          size="sm"
          renderItem={(place, index) => (
            <OrderRow
              place={place}
              index={index}
              facts={facts}
              mac={mac}
              onRemove={
                mac
                  ? null
                  : () =>
                      onChange(order.filter((held) => !samePlace(held, place)))
              }
            />
          )}
        />
      )}
      {mac ? null : (
        <AddPlace
          order={order}
          offered={offered}
          overflowHref={overflowHref}
          onAdd={(place) => onChange([...order, place])}
        />
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  order: { gap: 6 },
  row: {
    flex: 1,
    flexDirection: "row",
    alignItems: "center",
    gap: 10,
    minWidth: 0,
    paddingVertical: 8,
    paddingHorizontal: 10,
    borderWidth: 1,
    borderColor: colors.border,
    borderRadius: radii.md,
    backgroundColor: colors.surface,
  },
  index: {
    width: 20,
    height: 20,
    alignItems: "center",
    justifyContent: "center",
    borderWidth: 1,
    borderColor: colors.border2,
    borderRadius: radii.pill,
  },
  indexText: {
    fontFamily: fonts.sans,
    fontSize: 11,
    lineHeight: 14,
    fontVariant: ["tabular-nums"],
    color: colors.muted,
  },
  rowText: { flex: 1, gap: 2, minWidth: 0 },
  add: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 8,
    marginTop: 4,
  },
});
