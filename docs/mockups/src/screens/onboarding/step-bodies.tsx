// The open step of setup: the install command, the Boat key, and the first job.

import { CircleCheck } from "lucide-react";

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { TextField } from "../../components/fields.tsx";
import { CodeBlock } from "../../components/figures.tsx";
import {
  ConnectedMachine,
  InstallCommand,
  WaitingLine,
} from "../../components/install-command.tsx";
import { paths } from "../../components/paths.ts";
import { org } from "../../fixtures/org.ts";
import type { OnboardingState } from "./onboarding.tsx";

export function MachineStep({ state }: { state: OnboardingState }) {
  return (
    <div className="stack">
      <p className="small">
        Run this command on a Linux server. It installs the ciproxy agent and
        connects the server to {org.login}.
      </p>
      {state === "machine-no-kvm" ? (
        <Notice tone="danger" title="hetzner-ax42-1 cannot start VMs">
          ciproxy runs each job in its own VM, which needs KVM. Turn on
          virtualization for this server, then run the command again.
        </Notice>
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
      <TextField
        label="Boat API key"
        placeholder="boat_live_…"
        value={invalid ? "boat_live_3kQ9xV2" : undefined}
        hint={
          invalid
            ? undefined
            : "Create a key in the Boat console, under Settings."
        }
        error={
          invalid
            ? "Boat did not accept this key. Copy the whole key from the Boat console."
            : undefined
        }
        secret={invalid}
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
          <CircleCheck size={22} className="state-passed" />
          <div className="stack-tight">
            <p className="first-job-title">Your first job passed</p>
            <p className="small">
              <strong>test (rust)</strong> in {org.login}/mokly passed in 6m 12s
              on hetzner-ax42-1.
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
      <CodeBlock
        label=".github/workflows/ci.yml"
        lines={[
          "  jobs:",
          "    test:",
          "-     runs-on: ubuntu-latest",
          "+     runs-on: ciproxy",
        ]}
      />
      <WaitingLine>Waiting for a job with runs-on: ciproxy…</WaitingLine>
      <div className="row">
        <Action tone="ghost">See an example workflow</Action>
      </div>
    </div>
  );
}
