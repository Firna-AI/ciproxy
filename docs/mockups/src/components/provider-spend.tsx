// The spend of one provider in the month against its monthly limit, as
// Settings under Overflow and the Machines page show it: the costs that the
// provider confirmed and the costs that it has not confirmed yet, each with
// its own figure and never as one combined figure, as "What the limit
// counts" of overflow-spend.md says. The bar stacks both parts against the
// limit. A bound whose lifetime ended without a report stays with the
// unconfirmed costs.

import { formatMoney } from "../fixtures/format.ts";
import type { ProviderSpend } from "../fixtures/provider-spend.ts";
import { providerName, type ProviderKey } from "../fixtures/providers.ts";
import "./provider-spend.css";

export type SpendTone = "normal" | "warning";

function percent(share: number): string {
  return `${Math.round(share * 1000) / 10}%`;
}

function SpendBar({ spend, limit }: { spend: ProviderSpend; limit: number }) {
  const confirmed = Math.min(spend.confirmed / limit, 1);
  const unconfirmed = Math.min(spend.unconfirmed / limit, 1 - confirmed);
  return (
    <span
      className="spend-bar"
      role="img"
      aria-label={`${formatMoney(spend.confirmed)} confirmed and ${formatMoney(spend.unconfirmed)} not yet confirmed, of the ${formatMoney(limit)} limit`}
    >
      <span className="spend-confirmed" style={{ width: percent(confirmed) }} />
      <span
        className="spend-unconfirmed"
        style={{ width: percent(unconfirmed) }}
      />
    </span>
  );
}

function Part({
  kind,
  label,
  value,
}: {
  kind: "confirmed" | "unconfirmed";
  label: string;
  value: number;
}) {
  return (
    <div className="spend-part">
      <dt>
        <span className={`spend-key spend-key-${kind}`} aria-hidden="true" />
        {label}
      </dt>
      <dd className="num">{formatMoney(value)}</dd>
    </div>
  );
}

export function SpendFigures({
  provider,
  spend,
  limit,
  tone = "normal",
  compact = false,
}: {
  provider: ProviderKey;
  spend: ProviderSpend;
  /** The monthly spend limit of the provider, in US dollars. */
  limit: number;
  tone?: SpendTone;
  /** A row of a page, such as Machines, with no explanation below. */
  compact?: boolean;
}) {
  const name = providerName(provider);
  return (
    <div className={`provider-spend provider-spend-${tone}`}>
      <div className="spread">
        <span className="meter-label">Spent this month</span>
        <span className="meter-value num">Limit {formatMoney(limit)}</span>
      </div>
      <SpendBar spend={spend} limit={limit} />
      <dl className="spend-parts">
        <Part
          kind="confirmed"
          label={`Confirmed by ${name}`}
          value={spend.confirmed}
        />
        <Part
          kind="unconfirmed"
          label="Not yet confirmed"
          value={spend.unconfirmed}
        />
      </dl>
      {compact ? null : (
        <p className="caption">
          Not yet confirmed counts each start at its highest cost until {name}{" "}
          reports the cost.
          {spend.settled > 0
            ? ` It includes ${formatMoney(spend.settled)} for starts that ${name} did not report before their lifetime ended.`
            : ""}{" "}
          At the limit, ciproxy starts no new jobs on {name}.
        </p>
      )}
    </div>
  );
}
