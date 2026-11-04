// The closing call to action of a marketing page: one line to act on, what
// Free includes, and the two actions of the hero.

import { LinkAction } from "../../components/actions.tsx";
import { paths } from "../../components/paths.ts";
import { freePlanNote } from "../../fixtures/tiers.ts";
import { Actions, Section, SectionHead } from "./section.tsx";

export function ClosingCta() {
  return (
    <Section className="mk-closing">
      <SectionHead
        center
        title="Run your next job on your own machine"
        lead={freePlanNote}
      />
      <Actions center>
        <LinkAction to={paths.signIn} tone="primary" size="lg">
          Start free
        </LinkAction>
        <LinkAction to={paths.docsHome} size="lg">
          Read the docs
        </LinkAction>
      </Actions>
    </Section>
  );
}
