// The open step of setup: the install command, the Boat key with the monthly
// spend limit, and the first job. The first Admin adds the machine, also
// after a member asked for the install, because the Admin role alone adds a
// machine, as the answer to item 50 says. A machine that connected gets its
// first job image; while a first job waits for it, the step says why.

import { CircleCheck } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { BoatFields } from "../../components/boat-fields.tsx";
import { Notice } from "../../components/feedback.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import {
  ConnectedMachine,
  ImageNotice,
  InstallCommand,
  NoKvmNotice,
  WaitingLine,
} from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { unbroken } from "../../fixtures/format.ts";
import { ax42 } from "../../fixtures/hardware.ts";
import { boat } from "../../fixtures/machines.ts";
import { org, workspace } from "../../fixtures/org.ts";
import { runsOnDiff, workflowFile } from "../../fixtures/workflow.ts";
import type { OnboardingState } from "./onboarding.tsx";

/** The machine that Setup adds: the first machine of mokly-ai. */
export const setupMachine = "hetzner-ax42-1";

/** The first job while no runner has taken it. */
const firstJobWaiting = `test (rust) in ${org.login}/mokly is waiting for a runner…`;

export function MachineStep({ state }: { state: OnboardingState }) {
  const joined = state === "machine-connected" || state === "machine-preparing";
  return (
    <div className="stack">
      <p className="small">
        Run this command on a Linux machine. It installs the ciproxy agent and
        connects the machine to {workspace.name}.
      </p>
      {state === "machine-no-kvm" ? <NoKvmNotice name={setupMachine} /> : null}
      <InstallCommand />
      {joined ? (
        <>
          <ConnectedMachine
            name={setupMachine}
            hardware={ax42}
            preparing={state === "machine-preparing"}
          />
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

function SkipAction() {
  return (
    <LinkAction to={paths.firstJobAfterSkip} tone="ghost">
      Skip for now
    </LinkAction>
  );
}

/** The Boat step when no form can work: Boat is off or did not load. */
function BoatStepBlocked({ state }: { state: OnboardingState }) {
  const failed = state === "boat-load-failed";
  return (
    <div className="stack">
      <Notice
        tone={failed ? "danger" : "info"}
        title={failed ? "Unable to load Boat" : "Boat is unavailable right now"}
      >
        {failed
          ? "Check the connection, then try again."
          : "Connect Boat later in Settings under Overflow. Jobs run on your machines."}
      </Notice>
      <div className="row-wrap">
        {failed ? <Action tone="primary">Try again</Action> : null}
        <SkipAction />
      </div>
    </div>
  );
}

export function BoatStep({ state }: { state: OnboardingState }) {
  if (state === "boat-unavailable" || state === "boat-load-failed") {
    return <BoatStepBlocked state={state} />;
  }
  const rejected = state === "boat-rejected";
  const typed = state !== "boat";
  const checked = state === "boat-wallet" || state === "boat-connect-failed";
  return (
    <div className="stack">
      {rejected ? (
        <Notice
          tone="danger"
          title={`Boat rejected the API key on ${boat.keyRejectedOn}`}
        >
          Jobs run on your machines only until you connect Boat with a new key.
        </Notice>
      ) : (
        <p className="small">
          When your machines are busy, ciproxy starts jobs in your own Boat
          account. Boat bills you by the second.
        </p>
      )}
      <BoatFields
        invalid={state === "boat-invalid"}
        wallets={checked}
        reconnect={rejected}
        limit={typed ? boat.spendLimit : undefined}
      />
      {state === "boat-connect-failed" ? (
        <Notice tone="danger" title="Unable to connect Boat">
          Check the connection, then try Connect Boat again.
        </Notice>
      ) : null}
      <div className="row-wrap">
        <LinkAction
          to={checked ? paths.firstJob : paths.connectBoatWallet}
          tone="primary"
        >
          {checked ? "Connect Boat" : "Check key"}
        </LinkAction>
        <SkipAction />
      </div>
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
