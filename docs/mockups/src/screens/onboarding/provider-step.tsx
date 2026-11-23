// The overflow step of Setup: a choice of the providers of ciproxy, then the
// form of the picked one, with its key and the monthly spend limit of that
// provider. The Boat branch checks the key first and then asks for the
// wallet. A provider with no choice of billing account connects in one step.
// A provider that is not available shows as unavailable right now. Skip for
// now continues without overflow. After Boat rejected a saved key, the step
// asks for a new Boat key only.

import { Action, LinkAction } from "../../components/actions.tsx";
import { Notice } from "../../components/feedback.tsx";
import { paths } from "../../components/paths.ts";
import {
  connectIntro,
  ProviderChoice,
  SubmitKey,
} from "../../components/provider-choice.tsx";
import { ProviderFields } from "../../components/provider-fields.tsx";
import { boat } from "../../fixtures/overflow.ts";
import type { OnboardingState } from "./onboarding.tsx";

function SkipAction() {
  return (
    <LinkAction to={paths.firstJobAfterSkip} tone="ghost">
      Skip for now
    </LinkAction>
  );
}

/** No form can work: no provider is available, or the step did not load. */
function ProviderStepBlocked({ state }: { state: OnboardingState }) {
  const failed = state === "provider-load-failed";
  return (
    <div className="stack">
      <Notice
        tone={failed ? "danger" : "info"}
        title={
          failed
            ? "Unable to load the overflow providers"
            : "No overflow provider is available right now"
        }
      >
        {failed
          ? "Check the connection, then try again."
          : "Connect one later in Settings under Overflow. Jobs run on your machines."}
      </Notice>
      <div className="row-wrap">
        {failed ? <Action tone="primary">Try again</Action> : null}
        <SkipAction />
      </div>
    </div>
  );
}

/** Second provider is picked, also while Boat is unavailable. */
function SecondBranch({ boatUnavailable }: { boatUnavailable: boolean }) {
  return (
    <div className="stack">
      <p className="small">{connectIntro}</p>
      <ProviderChoice
        picked="second"
        unavailable={boatUnavailable ? ["boat"] : []}
      />
      <ProviderFields provider="second" />
      <div className="row-wrap">
        <SubmitKey provider="second" checked={false} to={paths.firstJob} />
        <SkipAction />
      </div>
    </div>
  );
}

export function ProviderStep({ state }: { state: OnboardingState }) {
  if (state === "no-provider-available" || state === "provider-load-failed") {
    return <ProviderStepBlocked state={state} />;
  }
  if (state === "second-provider" || state === "provider-unavailable") {
    return <SecondBranch boatUnavailable={state === "provider-unavailable"} />;
  }
  const rejected = state === "boat-rejected";
  const typed = state !== "provider";
  const checked = state === "boat-wallet" || state === "boat-connect-failed";
  return (
    <div className="stack">
      {rejected ? (
        <Notice
          tone="danger"
          title={`Boat rejected the API key on ${boat.keyRejectedOn}`}
        >
          ciproxy starts no new jobs on Boat until you connect a new key.
        </Notice>
      ) : (
        <>
          <p className="small">{connectIntro}</p>
          <ProviderChoice picked="boat" />
        </>
      )}
      <ProviderFields
        provider="boat"
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
        <SubmitKey
          provider="boat"
          checked={checked}
          to={checked ? paths.firstJob : paths.connectBoatWallet}
        />
        <SkipAction />
      </div>
    </div>
  );
}
