import { TextLink } from "../../../components/actions.tsx";
import { paths } from "../../../components/paths.ts";
import { costClaim } from "../../../fixtures/cost-claim.ts";
import { formatCount, formatMoneyCents } from "../../../fixtures/format.ts";
import { listPrices } from "../../../fixtures/price-lists.ts";
import { savingId } from "../saving-calculation.tsx";
import { StartActions, WorkflowLabel } from "./shared.tsx";
import "./monthly-estimate.css";

function EstimateSlip() {
  const claim = costClaim;
  return (
    <figure className="estimate-art">
      <div className="estimate-slip">
        <figcaption>
          <span className="slip-brand">ciproxy</span>
          <span className="study-kicker">Monthly estimate</span>
        </figcaption>
        <p className="slip-volume">
          {formatCount(claim.jobMinutes)} job minutes
        </p>
        <dl>
          <div>
            <dt>{claim.machine}</dt>
            <dd>{formatMoneyCents(listPrices.machine.amount)}</dd>
          </div>
          <div>
            <dt>ciproxy plan</dt>
            <dd>{formatMoneyCents(claim.plan.price)}</dd>
          </div>
          <div className="slip-total">
            <dt>Monthly total</dt>
            <dd>{formatMoneyCents(claim.withCiproxy)}</dd>
          </div>
          <div>
            <dt>On {claim.cheaper.name}</dt>
            <dd>{formatMoneyCents(claim.cheaper.cost)}</dd>
          </div>
        </dl>
        <p className="slip-saving">
          <span className="study-kicker">Monthly saving</span>
          <strong>{formatMoneyCents(claim.saved)}</strong>
        </p>
        <p className="slip-basis">
          One new {claim.machine}, busy for {claim.monthDays} days. Jobs average{" "}
          {claim.averageJobMinutes} minutes. Before tax and setup.
        </p>
        <p className="slip-link">
          <TextLink to={paths.pricing} fragment={savingId}>
            See the calculation
          </TextLink>
        </p>
      </div>
    </figure>
  );
}

export function MonthlyEstimate() {
  return (
    <section className="study estimate">
      <div className="study-inner study-split">
        <div className="estimate-copy">
          <WorkflowLabel />
          <h1>
            Same jobs.<span>Smaller bill.</span>
          </h1>
          <p className="study-lead">
            Put the GitHub Actions minutes you use every day on your own
            machines. Rent Boat runners only when those machines fill up.
          </p>
          <StartActions />
        </div>
        <EstimateSlip />
      </div>
    </section>
  );
}
