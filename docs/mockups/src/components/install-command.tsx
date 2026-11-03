// The one install command that connects a Linux server to the organization,
// shared by setup and Add machine.

import { CircleCheck } from "lucide-react";
import { Spinner } from "@firna/ui/spinner";

import { ax42 } from "../fixtures/machines.ts";
import { theme } from "../theme/firna.tsx";
import { CodeBlock } from "./figures.tsx";
import "./install-command.css";

export const installCommand =
  "curl -fsSL https://get.ciproxy.dev | sudo sh -s -- --token cpx_8Hq2Lw7Xr4Tz";

export function InstallCommand() {
  return (
    <div className="stack-tight">
      <CodeBlock label="On the server" lines={[installCommand]} />
      <p className="caption">
        The token works for one hour. The server needs x64 Linux with KVM, 4
        cores, and 16 GB of memory or more.
      </p>
    </div>
  );
}

export function WaitingLine({ children }: { children: string }) {
  return (
    <div className="waiting-line">
      <Spinner size={16} color={theme.colors.primary} />
      <span>{children}</span>
    </div>
  );
}

/** A machine that the agent just connected, with what it reported. */
export function ConnectedMachine({ name }: { name: string }) {
  return (
    <div className="connected-machine">
      <CircleCheck size={20} className="state-passed" />
      <div className="stack-tight">
        <p className="connected-title">{name} connected</p>
        <p className="caption">
          {ax42.threads} threads · {ax42.memoryGb} GB · KVM ready ·{" "}
          {ax42.slotsTotal} slots
        </p>
      </div>
    </div>
  );
}
