// The filters of History: the search, the place, and the date choice. The
// date choices are the history windows of the tiers that fit the history of
// the plan, in UTC days, with Today first. The filters stay above the list in
// every state, also when no job matches them.

import { Check, ChevronDown, Search } from "lucide-react";
import { InputFrame } from "@firna/ui/input";
import { SegmentedControl } from "@firna/ui/segmented";

import { Action, noop } from "../../components/actions.tsx";
import { dateChoices } from "../../fixtures/history-facts.ts";
import "./history-filters.css";

const whereOptions = [
  { label: "All", value: "all" },
  { label: "Own machines", value: "own" },
  { label: "Boat", value: "boat" },
] as const;

export type HistoryFilterValues = {
  search: string;
  place: (typeof whereOptions)[number]["value"];
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
      <SegmentedControl
        options={whereOptions}
        value={values.place}
        onChange={noop}
        accessibilityLabel="Where the job ran"
      />
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
