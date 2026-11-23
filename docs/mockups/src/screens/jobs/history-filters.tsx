// The filters of History: the search, the place, and the date choice. The
// place offers All, Own machines, and one choice for each provider that ran
// jobs in the window of the date choice. Its choices wrap on a narrow
// screen. The date choices are the history
// windows of the tiers that fit the history of the plan, in UTC days, with
// Today first. The filters stay above the list in every state, also when no
// job matches them.

import { Check, ChevronDown, Search } from "lucide-react";
import { InputFrame } from "@firna/ui/input";
import { SegmentedControl } from "@firna/ui/segmented";

import { Action, noop } from "../../components/actions.tsx";
import { dateChoices } from "../../fixtures/history-facts.ts";
import { providerName, type ProviderKey } from "../../fixtures/providers.ts";
import "./history-filters.css";

/** A place of the filter: every place, the own machines, or one provider. */
export type HistoryPlace = "all" | "own" | ProviderKey;

function placeOptions(providers: readonly ProviderKey[]) {
  return [
    { label: "All", value: "all" },
    { label: "Own machines", value: "own" },
    ...providers.map((key) => ({ label: providerName(key), value: key })),
  ] satisfies { label: string; value: HistoryPlace }[];
}

export type HistoryFilterValues = {
  search: string;
  place: HistoryPlace;
  /** The providers that ran jobs in the window, in the order of connection. */
  providers: readonly ProviderKey[];
  /** The date choice, such as "Today". */
  date: (typeof dateChoices)[number];
  /** The menu of the date choices is open. */
  menu?: boolean;
};

export function HistoryFilters({ values }: { values: HistoryFilterValues }) {
  return (
    <div className="history-filters">
      <span className="history-search">
        <InputFrame
          prefixIcon={Search}
          placeholder="Search jobs"
          value={values.search}
          onChangeText={noop}
          accessibilityLabel="Search jobs"
        />
      </span>
      <span className="history-place">
        <SegmentedControl<HistoryPlace>
          options={placeOptions(values.providers)}
          wrap
          value={values.place}
          onChange={noop}
          accessibilityLabel="Where the job ran"
        />
      </span>
      <span className="date-choice">
        <Action trailing={<ChevronDown size={16} />}>{values.date}</Action>
        {values.menu ? (
          <span className="date-menu" role="menu" aria-label="Finished">
            {dateChoices.map((choice) => (
              <span
                key={choice}
                className="date-menu-item"
                role="menuitemradio"
                aria-checked={choice === values.date}
              >
                <span>{choice}</span>
                {choice === values.date ? <Check size={16} /> : null}
              </span>
            ))}
          </span>
        ) : null}
      </span>
    </div>
  );
}
