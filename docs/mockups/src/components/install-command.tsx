// The one install command that connects a Linux machine to the workspace,
// shared by setup and Add machine, and the lines that follow it: the wait for
// the machine, the machine that connected, a machine without KVM, a machine
// whose disk holds fewer jobs than its slots, and the command after Revoke
// token. The command is one line: the install script of get.ciproxy.dev
// installs the agent, then runs it with the arguments after "--", as design
// call 30 says.

import type { ReactNode } from "react";
import { CircleCheck } from "lucide-react";

import { dotJoin, unbroken } from "../fixtures/format.ts";
import {
  hardwareSize,
  jobDiskBytes,
  machineSlots,
  type Hardware,
} from "../fixtures/hardware.ts";
import { defaultLabelJobs, labelNamed } from "../fixtures/settings.ts";
import { diskText, vmDiskBytes } from "../fixtures/vm-disk.ts";
import { Action, LinkAction } from "./actions.tsx";
import { Busy, Notice } from "./feedback.tsx";
import { CodeBlock } from "./figures.tsx";
import type { CataloguePath } from "./paths.ts";
import "./install-command.css";

export const installCommand =
  "curl -fsSL https://get.ciproxy.dev | sudo sh -s -- install --token cpx_8Hq2Lw7Xr4Tz";

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
          x64 Linux with KVM, {unbroken("4 cores")}, {unbroken("16 GB")} of
          memory, and {unbroken("250 GB")} of disk or more.
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

export function WaitingLine({ children }: { children: string }) {
  return (
    <div className="waiting-line">
      <Busy />
      <span>{children}</span>
    </div>
  );
}

/** A machine that the agent just connected, with what it reported. */
export function ConnectedMachine({
  name,
  hardware,
}: {
  name: string;
  hardware: Hardware;
}) {
  const { bySlots, atOnce } = defaultLabelJobs(hardware);
  return (
    <div className="connected-machine">
      <CircleCheck size={18} className="icon-ok" />
      <div className="stack-tight">
        <p className="connected-title">
          <span className="mono">{name}</span> connected
        </p>
        <p className="caption">
          {dotJoin(
            hardwareSize(hardware),
            unbroken("KVM ready"),
            unbroken(`${machineSlots(hardware)} slots`),
          )}
        </p>
        {atOnce < bySlots ? (
          <p className="caption">
            Its disk holds {atOnce} jobs of ciproxy at a time.
          </p>
        ) : null}
      </div>
    </div>
  );
}

/**
 * A machine whose disk holds fewer jobs of the label ciproxy than its slots:
 * each running job holds its VM disk, the image with the free disk of its
 * size, after the VM images and the host reserve.
 */
export function DiskLimitNotice({
  name,
  hardware,
}: {
  name: string;
  hardware: Hardware;
}) {
  const label = labelNamed("ciproxy");
  const { atOnce } = defaultLabelJobs(hardware);
  return (
    <Notice
      tone="warning"
      title={`Disk limits ${name} to ${atOnce} jobs of ciproxy at a time`}
    >
      With the current image, a job of ciproxy holds{" "}
      {diskText(vmDiskBytes(label))} of disk. The machine has{" "}
      {diskText(jobDiskBytes(hardware))} for jobs after the VM images and the
      host reserve. Add disk to run more jobs at once.
    </Notice>
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
