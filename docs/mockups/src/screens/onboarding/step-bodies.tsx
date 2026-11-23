// The open step of setup: the install command and the first job. The
// provider step is in provider-step.tsx. The first Admin adds the machine,
// also after a member asked for the install, because the Admin role alone
// adds a machine, as the answer to item 50 says. A machine that connected
// gets its first job image; while a first job waits for it, the step says
// why. A Mac connects with the same command, and the step says that it runs
// the jobs of macOS labels.

import { CircleCheck } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import {
  ConnectedMachine,
  ImageNotice,
  InstallCommand,
  NoVirtualizationNotice,
  WaitingLine,
} from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { ax42 } from "../../fixtures/hardware.ts";
import { macStudioMachine } from "../../fixtures/macs.ts";
import { org, workspace } from "../../fixtures/org.ts";
import { runsOnDiff, workflowFile } from "../../fixtures/workflow.ts";
import type { OnboardingState } from "./onboarding.tsx";

/** The machine that Setup adds: the first machine of mokly-ai. */
export const setupMachine = "hetzner-ax42-1";

/** The first job while no runner has taken it. */
const firstJobWaiting = `test (rust) in ${org.login}/mokly is waiting for a runner…`;

export function MachineStep({ state }: { state: OnboardingState }) {
  const mac = state === "machine-mac";
  const joined =
    state === "machine-connected" || state === "machine-preparing" || mac;
  return (
    <div className="stack">
      <p className="small">
        Run this command on a Linux machine or a Mac. It installs the ciproxy
        agent and connects the machine to {workspace.name}.
      </p>
      {state === "machine-no-kvm" ? (
        <NoVirtualizationNotice name={setupMachine} />
      ) : null}
      <InstallCommand />
      {joined ? (
        <>
          <ConnectedMachine
            name={mac ? macStudioMachine.name : setupMachine}
            hardware={mac ? macStudioMachine.hardware : ax42}
            preparing={state === "machine-preparing"}
          />
          <div className="row">
            <LinkAction to={paths.connectProvider} tone="primary">
              Continue
            </LinkAction>
          </div>
        </>
      ) : null}
      {state === "machine-waiting" ? (
        <WaitingLine>Waiting for the machine to connect…</WaitingLine>
      ) : null}
    </div>
  );
}

function FirstJobPassed() {
  return (
    <div className="stack">
      <div className="first-job">
        <CircleCheck size={18} className="icon-ok" />
        <div className="stack-tight">
          <p className="first-job-title">Your first job passed</p>
          <p className="small">
            <strong>test (rust)</strong> in {org.login}/mokly passed in{" "}
            {unbroken("6m 12s")} on <span className="mono">{setupMachine}</span>
            .
          </p>
        </div>
      </div>
      <div className="row-wrap">
        <LinkAction to={paths.jobs} tone="primary">
          Open Jobs
        </LinkAction>
        <LinkAction to={paths.addMachine} tone="secondary">
          Add another machine
        </LinkAction>
      </div>
    </div>
  );
}

/**
 * The first job, before it arrives, while it waits for the machine to get its
 * first job image, and after it passed.
 */
export function FirstJobStep({ state }: { state: OnboardingState }) {
  if (state === "first-job-passed") return <FirstJobPassed />;
  return (
    <div className="stack">
      <p className="small">
        Change runs-on in one workflow file, then push the change.
      </p>
      <CodeBlock label={workflowFile} lines={runsOnDiff} />
      {state === "first-job-preparing" ? (
        <>
          <WaitingLine>{firstJobWaiting}</WaitingLine>
          <ImageNotice name={setupMachine}>
            The job starts on it when it is ready.
          </ImageNotice>
        </>
      ) : (
        <>
          <WaitingLine>Waiting for a job with runs-on: ciproxy…</WaitingLine>
          <div className="row">
            <Action size="sm">See an example workflow</Action>
          </div>
        </>
      )}
    </div>
  );
}
