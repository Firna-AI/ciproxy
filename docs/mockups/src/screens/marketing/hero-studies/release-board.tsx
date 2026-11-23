import { useId } from "react";
import { Activity } from "lucide-react";

import { formatDay } from "../../../fixtures/format.ts";
import { releaseDay } from "../../../fixtures/live-jobs.ts";
import { org } from "../../../fixtures/org.ts";
import { liveScenes, type LiveScene } from "../../jobs/live-scenes.ts";
import { StartActions, WorkflowLabel } from "./shared.tsx";
import "./release-board.css";

function BoardTotal({ label, count }: { label: string; count: number }) {
  return (
    <div className="board-total">
      <dt className="study-kicker">{label}</dt>
      <dd>{String(count).padStart(2, "0")}</dd>
    </div>
  );
}

function Snapshot({ scene, name }: { scene: LiveScene; name: string }) {
  const ownJob = scene.runningJobs.find((job) => job.where.kind === "own");
  const boatJob = scene.runningJobs.find((job) => job.where.kind === "boat");
  return (
    <div className={`board-snapshot board-${name}`}>
      <dl className="board-totals">
        <BoardTotal label="On your machines" count={scene.ownBusy} />
        <BoardTotal label="On Boat" count={scene.onBoat} />
        <BoardTotal label="Waiting" count={scene.waiting} />
      </dl>
      <ul className="board-runs">
        {[ownJob, boatJob].map(
          (job) =>
            job && (
              <li key={job.id}>
                <span>{job.name}</span>
                <span className="board-ref">
                  {job.repo} / {job.ref}
                </span>
                <span>
                  {job.where.kind === "own" ? job.where.machine : "Boat"}
                </span>
                <span className="board-elapsed">{job.time}</span>
              </li>
            ),
        )}
      </ul>
    </div>
  );
}

export function ReleaseBoard() {
  const toggle = useId();
  return (
    <section className="study dispatch">
      <div className="study-inner">
        <div className="dispatch-top">
          <div>
            <WorkflowLabel />
            <h1>
              Full machines?<span>Keep shipping.</span>
            </h1>
          </div>
          <div>
            <p className="study-lead">
              Your machines run GitHub Actions first. At release time, ciproxy
              sends the overflow to your Boat account.
            </p>
            <StartActions />
          </div>
        </div>
        <fieldset className="dispatch-board">
          <legend className="board-legend">Release day job counts</legend>
          <div className="board-head">
            <h2>
              <Activity size={20} aria-hidden="true" /> Release board
            </h2>
            <span className="study-kicker">
              {org.login} / {formatDay(releaseDay)}
            </span>
            <div className="board-switch">
              <label>
                <input type="radio" name={toggle} value="peak" defaultChecked />
                Release peak
              </label>
              <label>
                <input type="radio" name={toggle} value="steady" />
                Afternoon
              </label>
            </div>
          </div>
          <Snapshot scene={liveScenes.peak} name="peak" />
          <Snapshot scene={liveScenes.steady} name="steady" />
        </fieldset>
      </div>
    </section>
  );
}
