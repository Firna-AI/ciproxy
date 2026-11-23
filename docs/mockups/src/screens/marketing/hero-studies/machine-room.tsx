import { Cloud, Cpu } from "lucide-react";

import { machineScenes } from "../../../fixtures/machines.ts";
import { org } from "../../../fixtures/org.ts";
import { liveScenes } from "../../jobs/live-scenes.ts";
import { RackDrawing } from "./rack-drawing.tsx";
import { StartActions, WorkflowLabel } from "./shared.tsx";
import "./machine-room.css";

export function MachineRoom() {
  const { ownBusy, ownSlots, onBoat } = liveScenes.peak;
  return (
    <section className="study room">
      <div className="study-inner study-split">
        <div>
          <WorkflowLabel />
          <h1>
            Your machines <span>get first pick.</span>
          </h1>
          <p className="study-lead">
            Run GitHub Actions on the machines you already pay for. When they
            fill up, ciproxy starts runners in your Boat account.
          </p>
          <StartActions />
        </div>
        <figure className="room-plate">
          <figcaption className="room-meta study-kicker">
            <span>
              <Cpu size={16} /> Machine pool
            </span>
            <span>{org.login}</span>
          </figcaption>
          <div className="room-drawing">
            <p className="room-model">
              Hetzner AX42 <sup>× {machineScenes.peak.length}</sup>
            </p>
            <RackDrawing />
          </div>
          <div className="room-slots">
            <span className="study-kicker">Own slots busy</span>
            <strong>
              {ownBusy} / {ownSlots}
            </strong>
            <div className="room-lights" aria-hidden="true">
              {machineScenes.peak.map((machine) => (
                <span key={machine.name}>
                  {machine.slots.map((slot, i) => (
                    <i key={i} data-slot={slot} />
                  ))}
                </span>
              ))}
            </div>
          </div>
          <p className="room-handoff">
            <span>
              <Cloud size={18} /> Boat overflow
            </span>
            <strong>{onBoat} running</strong>
          </p>
        </figure>
      </div>
    </section>
  );
}
