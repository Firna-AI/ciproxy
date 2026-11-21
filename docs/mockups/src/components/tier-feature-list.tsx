// The features of a tier as a list: a check for each feature that ships at
// launch, and a clock with "Coming later" for each feature that comes later.
// The pricing page of the site and the Plans page of the dashboard share it,
// and the comparison table marks its rows with the same words.

import { Check, Clock } from "lucide-react";

import type { TierListItem } from "../fixtures/tiers.ts";
import "./tier-feature-list.css";

/** The mark of a feature that a tier lists but that ships after launch. */
export function ComingLater() {
  return <span className="coming-later">Coming later</span>;
}

export function TierFeatureList({
  items,
  size = "md",
}: {
  items: readonly TierListItem[];
  /** The small list of the dashboard cards, or the list of the site. */
  size?: "md" | "sm";
}) {
  return (
    <ul className={`tier-feature-list is-${size}`}>
      {items.map((item) => {
        const Icon = item.later ? Clock : Check;
        return (
          <li key={item.name} className={item.later ? "is-later" : undefined}>
            <Icon size={14} className="tier-feature-icon" aria-hidden="true" />
            <span className="tier-feature-text">
              <span>{item.name}</span>
              {item.later ? <ComingLater /> : null}
            </span>
          </li>
        );
      })}
    </ul>
  );
}
