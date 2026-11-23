import { GitBranch } from "lucide-react";

import { LinkAction } from "../../../components/actions.tsx";
import { paths } from "../../../components/paths.ts";
import { defaultLabel } from "../../../fixtures/jobs.ts";
import { freePlanNote } from "../../../fixtures/tiers.ts";
import { Actions } from "../section.tsx";
import "./shared.css";

export function WorkflowLabel() {
  return (
    <p className="study-workflow">
      <GitBranch size={15} aria-hidden="true" />
      <code>runs-on: {defaultLabel}</code>
    </p>
  );
}

export function StartActions() {
  return (
    <div className="study-start">
      <Actions>
        <LinkAction to={paths.signIn} tone="primary" size="lg">
          Start free
        </LinkAction>
        <LinkAction to={paths.docsHome} size="lg">
          Read the docs
        </LinkAction>
      </Actions>
      <p className="caption">{freePlanNote}</p>
    </div>
  );
}
