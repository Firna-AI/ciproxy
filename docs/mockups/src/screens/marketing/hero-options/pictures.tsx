// The pictures of the hero options, each from the fixtures of mokly-ai at the
// release peak: the machines with their slots and the Boat overflow, a monthly
// estimate from the cost claim, and a board of the running and waiting jobs.

import { Cloud } from "lucide-react";

import { TextLink } from "../../../components/actions.tsx";
import { SlotStrip } from "../../../components/figures.tsx";
import { paths } from "../../../components/paths.ts";
import {
  DotLabel,
  JobStateDot,
  WhereLabel,
} from "../../../components/status.tsx";
import { costClaim } from "../../../fixtures/cost-claim.ts";
import { formatCount, formatMoneyCents } from "../../../fixtures/format.ts";
import { boat, machineScenes } from "../../../fixtures/machines.ts";
import { org } from "../../../fixtures/org.ts";
import { listPrices } from "../../../fixtures/price-lists.ts";
import { liveScenes } from "../../jobs/live-scenes.ts";
import { savingId } from "../saving-calculation.tsx";
import { RackDrawing } from "./rack-drawing.tsx";
import "./hero-pictures.css";

const peak = liveScenes.peak;

/** The bar of a card: the organization, the page, and the live dot. */
function CardBar({ page }: { page: string }) {
  return (
    <div className="ho-bar">
      <span className="ho-bar-org">
        <span className="strong">{org.login}</span>
        <span className="ho-slash" aria-hidden="true">
          /
        </span>
        <span className="muted">{page}</span>
      </span>
      <DotLabel tone="run">Live</DotLabel>
    </div>
  );
}

/** The machines with their busy slots, and the jobs that run on Boat. */
export function MachinesCard({ drawing = false }: { drawing?: boolean }) {
  return (
    <figure className="ho-card" aria-label="The machines of mokly-ai">
      <CardBar page="Machines" />
      {drawing && (
        <div className="ho-drawing">
          <RackDrawing />
        </div>
      )}
      <div className="ho-body">
        <p className="ho-label">
          <span>Your machines</span>
          <span className="num muted">
            {peak.ownBusy} of {peak.ownSlots} slots busy
          </span>
        </p>
        <ul className="ho-list">
          {machineScenes.peak.map((machine) => (
            <li key={machine.name} className="ho-machine">
              <span className="mono">{machine.name}</span>
              <SlotStrip slots={machine.slots} size="lg" />
            </li>
          ))}
        </ul>
        <p className="ho-boat">
          <span className="ho-boat-name">
            <Cloud size={15} strokeWidth={1.75} />
            <span className="strong">Boat</span>
          </span>
          <span className="num">{formatCount(peak.onBoat)} running</span>
        </p>
        <p className="caption">
          {peak.waiting} jobs wait for a slot. Each starts on Boat after{" "}
          {boat.waitSeconds} seconds.
        </p>
      </div>
    </figure>
  );
}

/** One row of the estimate: a name and its monthly price. */
function EstimateRow({ name, cost }: { name: string; cost: number }) {
  return (
    <div className="ho-row">
      <dt>{name}</dt>
      <dd className="num">{formatMoneyCents(cost)}</dd>
    </div>
  );
}

/** The monthly cost of one busy machine with ciproxy, against the cheaper
 * hosted runners, from the cost claim of the landing page. */
export function EstimateCard() {
  const claim = costClaim;
  return (
    <figure className="ho-card ho-estimate" aria-label="A monthly estimate">
      <figcaption className="ho-estimate-head">
        <span className="mk-eyebrow">Monthly estimate</span>
        <span className="mono muted">
          {formatCount(claim.jobMinutes)} job minutes
        </span>
      </figcaption>
      <dl className="ho-rows">
        <EstimateRow name={claim.machine} cost={listPrices.machine.amount} />
        <EstimateRow name="ciproxy plan" cost={claim.plan.price} />
        <div className="ho-row is-total">
          <dt>With ciproxy</dt>
          <dd className="num">{formatMoneyCents(claim.withCiproxy)}</dd>
        </div>
        <div className="ho-row is-other">
          <dt>On {claim.cheaper.name}</dt>
          <dd className="num">{formatMoneyCents(claim.cheaper.cost)}</dd>
        </div>
      </dl>
      <div className="ho-saving">
        <span className="mk-eyebrow">You save</span>
        <strong className="num">{formatMoneyCents(claim.saved)}</strong>
        <span className="muted">a month</span>
      </div>
      <p className="caption ho-basis">
        <span>
          One new {claim.machine}, busy for {claim.monthDays} days, with jobs of{" "}
          {claim.averageJobMinutes} minutes on average.
        </span>
        <TextLink to={paths.pricing} fragment={savingId}>
          See the calculation
        </TextLink>
      </p>
    </figure>
  );
}

/** The release peak as three counts, with the first running jobs. */
export function PeakBoard({ rows }: { rows: number }) {
  const counts = [
    { label: "On your machines", value: peak.ownBusy },
    { label: "On Boat", value: peak.onBoat },
    { label: "Waiting", value: peak.waiting },
  ];
  return (
    <figure className="ho-card ho-board" aria-label="The jobs of mokly-ai">
      <CardBar page="Jobs" />
      <dl className="ho-counts">
        {counts.map((count) => (
          <div key={count.label} className="ho-count">
            <dt className="mk-eyebrow">{count.label}</dt>
            <dd className="num">{count.value}</dd>
          </div>
        ))}
      </dl>
      <ul className="ho-list ho-jobs">
        {peak.runningJobs.slice(0, rows).map((job) => (
          <li key={job.id} className="ho-job">
            <JobStateDot state={job.state} />
            <span className="ho-job-name">{job.name}</span>
            <span className="ho-job-ref muted">
              {job.repo} {job.ref}
            </span>
            <WhereLabel where={job.where} />
            <span className="ho-job-time num muted">{job.time}</span>
          </li>
        ))}
      </ul>
    </figure>
  );
}
