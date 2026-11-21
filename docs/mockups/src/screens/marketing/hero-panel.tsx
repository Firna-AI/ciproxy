// A picture of the product for the hero: the live queue of mokly-ai at a
// release peak. Every slot of its three machines is busy, the overflow runs on
// Boat, and the running jobs show where each one runs.

import { Cloud } from "lucide-react";
import type { Viewport } from "@mokly/mokly";
import { Avatar } from "@firna/ui/avatar";

import { Meter, SlotStrip } from "../../components/figures.tsx";
import { DotLabel, JobStateDot, WhereLabel } from "../../components/status.tsx";
import { formatCount, formatMoney } from "../../fixtures/format.ts";
import { boat, machineScenes } from "../../fixtures/machines.ts";
import { org } from "../../fixtures/org.ts";
import { liveScenes } from "../jobs/live-scenes.ts";
import "./hero-panel.css";

const peak = liveScenes.peak;

export function HeroPanel({ viewport }: { viewport: Viewport }) {
  const jobs = peak.runningJobs.slice(0, viewport === "mobile" ? 4 : 7);
  return (
    <figure className="hero-panel" aria-label="The live queue of mokly-ai">
      <div className="hp-bar">
        <span className="hp-org">
          <Avatar
            label={org.initial}
            accessibilityLabel={org.login}
            size={20}
            shape="square"
          />
          <span className="strong">{org.login}</span>
          <span className="hp-slash" aria-hidden="true">
            /
          </span>
          <span className="muted">Jobs</span>
        </span>
        <DotLabel tone="run">Live</DotLabel>
      </div>
      <div className="hp-body">
        <div className="hp-machines">
          <p className="hp-label">
            <span>Your machines</span>
            <span className="num muted">
              {peak.ownBusy} of {peak.ownSlots} slots busy
            </span>
          </p>
          <ul className="hp-list">
            {machineScenes.peak.map((machine) => (
              <li key={machine.name} className="hp-machine">
                <span className="mono hp-machine-name">{machine.name}</span>
                <SlotStrip slots={machine.slots} size="lg" />
              </li>
            ))}
          </ul>
          <div className="hp-boat">
            <span className="hp-boat-name">
              <Cloud size={15} strokeWidth={1.75} className="icon-muted" />
              <span className="strong">Boat</span>
            </span>
            <span className="num">{formatCount(peak.onBoat)} running</span>
          </div>
          <p className="caption">
            {peak.waiting} jobs wait for a slot. Each starts on Boat after{" "}
            {boat.waitSeconds} seconds.
          </p>
          <div className="hp-spend">
            <Meter
              label="Boat this month"
              used={boat.spentThisMonth}
              max={boat.spendLimit}
              value={`${formatMoney(boat.spentThisMonth)} of ${formatMoney(boat.spendLimit)}`}
            />
          </div>
        </div>
        <div className="hp-jobs">
          <p className="hp-label">
            <span>Running</span>
            <span className="num muted">{formatCount(peak.running)}</span>
          </p>
          <ul className="hp-list">
            {jobs.map((job) => (
              <li key={job.id} className="hp-job">
                <JobStateDot state={job.state} />
                <span className="hp-job-name">{job.name}</span>
                <span className="hp-job-ref muted">
                  {job.repo} {job.ref}
                </span>
                <WhereLabel where={job.where} />
                <span className="hp-job-time num muted">{job.time}</span>
              </li>
            ))}
          </ul>
        </div>
      </div>
    </figure>
  );
}
