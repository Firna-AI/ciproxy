// The one install command that connects a Linux machine to the organization,
// shared by setup and Add machine, and the lines that follow it: the wait for
// the machine, the machine that connected, a machine without KVM, and the
// command after Revoke token. An Admin whom GitHub does not let manage
// self-hosted runners sees who can add a machine in place of the command.

import type { ReactNode } from "react";
import { CircleCheck } from "lucide-react";

import { dotJoin, unbroken } from "../fixtures/format.ts";
import { ax42, ax42Size } from "../fixtures/machines.ts";
import { Action, LinkAction } from "./actions.tsx";
import { Busy, Notice } from "./feedback.tsx";
import { CodeBlock } from "./figures.tsx";
import type { CataloguePath } from "./paths.ts";
import "./install-command.css";

export const installCommand =
  "curl -fsSL https://get.ciproxy.dev | sudo sh -s -- --token cpx_8Hq2Lw7Xr4Tz";

export function InstallCommand({
  revokeTo,
}: {
  /** The page after Revoke token, where the catalogue has one. */
  revokeTo?: CataloguePath;
}) {
  return (
    <div className="stack-tight">
      <CodeBlock label="On the machine" lines={[installCommand]} />
      <div className="install-note">
        <p className="caption">
          The token adds any number of machines for one hour. A machine needs
          x64 Linux with KVM, {unbroken("4 cores")}, and {unbroken("16 GB")} of
          memory or more.
        </p>
        {revokeTo ? (
          <LinkAction to={revokeTo} size="sm" tone="ghost">
            Revoke token
          </LinkAction>
        ) : (
          <Action size="sm" tone="ghost">
            Revoke token
          </Action>
        )}
      </div>
    </div>
  );
}

/** The command after Revoke token: its token adds no more machines. */
export function RevokedCommand({ next }: { next: CataloguePath }) {
  return (
    <Notice
      tone="info"
      title="Token revoked"
      actions={
        <LinkAction to={next} tone="primary" size="sm">
          Create a new command
        </LinkAction>
      }
    >
      The install command adds no more machines. The machines that joined with
      it stay connected.
    </Notice>
  );
}

/** For an Admin without the GitHub runner right: who can add a machine. */
export function NoRunnerRightNotice() {
  return (
    <Notice
      tone="info"
      title="Ask an owner to add machines"
      actions={<Action size="sm">Check again</Action>}
    >
      An owner of the organization, or an Admin with the GitHub runner right,
      can add a machine. An owner can also give you that right.
    </Notice>
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

/** A machine that connected without KVM: it gets no slots until it has KVM. */
export function NoKvmNotice({
  name,
  actions,
}: {
  name: string;
  actions?: ReactNode;
}) {
  return (
    <Notice tone="danger" title={`${name} cannot start VMs`} actions={actions}>
      ciproxy runs each job in its own VM, which needs KVM. Turn on
      virtualization for this machine, then run the command again.
    </Notice>
  );
}
