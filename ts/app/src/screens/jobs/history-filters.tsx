// The filters of History: the search, the place, and the date choice. Each
// change replaces the address, so a reload keeps the filters. The search
// applies after a short pause or at Enter. The date choices are Today and
// each history window of the tiers up to the history of the plan.

import { useEffect, useRef, useState } from "react";
import { StyleSheet, View } from "react-native";
import { Button } from "@firna/ui/button";
import { DropdownMenu, type DropdownListEntry } from "@firna/ui/dropdown";
import { InputFrame } from "@firna/ui/input";
import { SegmentedControl } from "@firna/ui/segmented";

import { ChevronDownIcon, SearchIcon } from "../../components/icons.tsx";
import { useServices } from "../../services/context.tsx";
import {
  jobsHref,
  windowChoices,
  windowLabels,
  type HistoryFilters,
} from "../../view/jobs-links.ts";

/** The pause after the last key before a search applies. */
export const searchPause = 400;

const places = [
  { label: "All", value: "all" },
  { label: "Own machines", value: "machine" },
  { label: "Boat", value: "boat" },
] as const;

type PlaceValue = (typeof places)[number]["value"];

export function HistoryFiltersView({
  workspaceId,
  filters,
  historyDays,
}: {
  workspaceId: string;
  filters: HistoryFilters;
  /** The history of the plan in days, once the API gave it. */
  historyDays: number | null;
}) {
  const { navigator, env } = useServices();
  const [typed, setTyped] = useState(filters.search);
  const pending = useRef<(() => void) | null>(null);
  // A search that applies after the pause uses the filters of that moment,
  // so a place or date chosen during the pause stays.
  const latest = useRef(filters);
  latest.current = filters;

  useEffect(() => setTyped(filters.search), [filters.search]);
  useEffect(() => () => pending.current?.(), []);

  const apply = (next: HistoryFilters) =>
    navigator.go(jobsHref(workspaceId, "history", next), { replace: true });

  const searchNow = (value: string) => {
    pending.current?.();
    pending.current = null;
    const current = latest.current;
    if (value.trim() !== current.search.trim()) {
      apply({ ...current, search: value });
    }
  };

  const onType = (value: string) => {
    setTyped(value);
    pending.current?.();
    pending.current = env.timers.schedule(() => searchNow(value), searchPause);
  };

  const entries: DropdownListEntry[] = windowChoices(historyDays ?? 1).map(
    (days) => ({
      id: `days-${days}`,
      label: windowLabels[days],
      type: "item",
      selected: days === filters.days,
      onPress: () => apply({ ...filters, days }),
    }),
  );

  return (
    <View style={styles.row}>
      <View style={styles.search}>
        <InputFrame
          prefixIcon={SearchIcon}
          placeholder="Search jobs"
          value={typed}
          onChangeText={onType}
          onSubmitEditing={() => searchNow(typed)}
          accessibilityLabel="Search jobs"
          returnKeyType="search"
        />
      </View>
      <SegmentedControl<PlaceValue>
        options={places}
        value={filters.place ?? "all"}
        onChange={(value) =>
          apply({ ...filters, place: value === "all" ? null : value })
        }
        accessibilityLabel="Where the job ran"
      />
      <DropdownMenu
        entries={entries}
        accessibilityLabel="Finished"
        align="end"
        minWidth={200}
      >
        <Button
          tone="secondary"
          accessibilityLabel={`Finished: ${windowLabels[filters.days]}`}
          trailing={<ChevronDownIcon size={16} />}
        >
          {windowLabels[filters.days]}
        </Button>
      </DropdownMenu>
    </View>
  );
}

const styles = StyleSheet.create({
  row: {
    flexDirection: "row",
    flexWrap: "wrap",
    alignItems: "center",
    gap: 10,
  },
  search: { flexGrow: 1, flexBasis: 220, minWidth: 220 },
});
