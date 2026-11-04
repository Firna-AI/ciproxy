// The open step of setup: the install command, the Boat key with the monthly
// spend limit, and the first job. Only an Admin whom GitHub lets manage
// self-hosted runners sees the install command; another Admin sees who can
// add the machine.

import { CircleCheck } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { BoatFields } from "../../components/boat-fields.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import {
  ConnectedMachine,
  InstallCommand,
  NoKvmNotice,
  NoRunnerRightNotice,
  WaitingLine,
} from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { boat } from "../../fixtures/machines.ts";
import { org } from "../../fixtures/org.ts";
import { runsOnDiff, workflowFile } from "../../fixtures/workflow.ts";
import type { OnboardingState } from "./onboarding.tsx";

/** The first Admin asked for the install, so GitHub gives no runner right. */
function NoRunnerRight() {
  return (
    <div className="stack">
      <NoRunnerRightNotice />
      <div className="row">
        <LinkAction to={paths.connectBoat}>Skip for now</LinkAction>
      </div>
    </div>
  );
}

export function MachineStep({ state }: { state: OnboardingState }) {
  if (state === "machine-no-right") return <NoRunnerRight />;
  return (
    <div className="stack">
      <p className="small">
        Run this command on a Linux machine. It installs the ciproxy agent and
        connects the machine to {org.login}.
      </p>
      {state === "machine-no-kvm" ? (
        <NoKvmNotice name="hetzner-ax42-1" />
      ) : null}
      <InstallCommand />
      {state === "machine-connected" ? (
        <>
          <ConnectedMachine name="hetzner-ax42-1" />
          <div className="row">
            <LinkAction to={paths.connectBoat} tone="primary">
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

export function BoatStep({ invalid }: { invalid: boolean }) {
  return (
    <div className="stack">
      <p className="small">
        When your machines are busy, ciproxy starts jobs in your own Boat
        account. Boat bills you by the second.
      </p>
      <BoatFields
        invalid={invalid}
        limit={invalid ? boat.spendLimit : undefined}
      />
      <div className="row-wrap">
        <LinkAction to={paths.firstJob} tone="primary">
          Connect Boat
        </LinkAction>
        <LinkAction to={paths.firstJob} tone="ghost">
          Skip for now
        </LinkAction>
      </div>
    </div>
  );
}

export function FirstJobStep({ passed }: { passed: boolean }) {
  if (passed) {
    return (
      <div className="stack">
        <div className="first-job">
          <CircleCheck size={18} className="icon-ok" />
          <div className="stack-tight">
            <p className="first-job-title">Your first job passed</p>
            <p className="small">
              <strong>test (rust)</strong> in {org.login}/mokly passed in{" "}
              {unbroken("6m 12s")} on{" "}
              <span className="mono">hetzner-ax42-1</span>.
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
  return (
    <div className="stack">
      <p className="small">
        Change runs-on in one workflow file, then push the change.
      </p>
      <CodeBlock label={workflowFile} lines={runsOnDiff} />
      <WaitingLine>Waiting for a job with runs-on: ciproxy…</WaitingLine>
      <div className="row">
        <Action size="sm">See an example workflow</Action>
      </div>
    </div>
  );
}
