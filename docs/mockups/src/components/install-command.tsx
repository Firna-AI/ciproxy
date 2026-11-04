// The one install command that connects a Linux machine to the organization,
// shared by setup and Add machine, and the lines that follow it: the wait for
// the machine, and the machine that connected.

import { CircleCheck } from "lucide-react";

import { dotJoin, unbroken } from "../fixtures/format.ts";
import { ax42, ax42Size } from "../fixtures/machines.ts";
import { Busy } from "./feedback.tsx";
import { CodeBlock } from "./figures.tsx";
import "./install-command.css";

export const installCommand =
  "curl -fsSL https://get.ciproxy.dev | sudo sh -s -- --token cpx_8Hq2Lw7Xr4Tz";

export function InstallCommand() {
  return (
    <div className="stack-tight">
      <CodeBlock label="On the machine" lines={[installCommand]} />
      <p className="caption">
        The token works for one hour. The machine needs x64 Linux with KVM, 4
        cores, and 16 GB of memory or more.
      </p>
    </div>
  );
}

export function WaitingLine({ children }: { children: string }) {
  return (
    <div className="waiting-line">
      <Busy />
      <span>{children}</span>
    </div>
  );
}

/** A machine that the agent just connected, with what it reported. */
export function ConnectedMachine({ name }: { name: string }) {
  return (
    <div className="connected-machine">
      <CircleCheck size={18} className="icon-ok" />
      <div className="stack-tight">
        <p className="connected-title">
          <span className="mono">{name}</span> connected
        </p>
        <p className="caption">
          {dotJoin(
            ax42Size,
            unbroken("KVM ready"),
            unbroken(`${ax42.slotsTotal} slots`),
          )}
        </p>
      </div>
    </div>
  );
}
