// The actions of a page model, in order. A link out of the app is an
// anchor; an install link first marks this tab, so the return from GitHub can
// show what the install did. A page of the app opens through the router.

import { Link } from "expo-router";

import { useServices } from "../services/context.tsx";
import type { PageAction } from "../view/page-model.ts";
import { ActionButton, ActionLink, type Size } from "./actions.tsx";
import { ExternalIcon } from "./icons.tsx";

export type ActionHandlers = {
  onCheckAgain?: () => void;
  onClaim?: (workspaceId: string) => void;
  /** The action that runs now, which shows busy. */
  busy?: PageAction["kind"] | null;
};

export function PageActionView({
  action,
  handlers,
  size = "lg",
  block = true,
}: {
  action: PageAction;
  handlers: ActionHandlers;
  size?: Size;
  block?: boolean;
}) {
  const services = useServices();
  switch (action.kind) {
    case "leave":
      return (
        <ActionLink
          label={action.label}
          href={action.href}
          tone={action.tone}
          size={size}
          block={block}
          newTab={action.newTab}
          icon={action.icon === "external" ? ExternalIcon : undefined}
          onClick={
            action.install ? () => services.markInstallStarted() : undefined
          }
        />
      );
    case "go":
      return (
        <Link href={action.href} asChild>
          <ActionLink
            label={action.label}
            href={action.href}
            tone={action.tone}
            size={size}
            block={block}
          />
        </Link>
      );
    case "check-again":
      return (
        <ActionButton
          label={action.label}
          tone={action.tone}
          size={size}
          block={block}
          busy={handlers.busy === "check-again"}
          onPress={() => handlers.onCheckAgain?.()}
        />
      );
    case "claim-admin":
      return (
        <ActionButton
          label={action.label}
          tone={action.tone}
          size={size}
          block={block}
          busy={handlers.busy === "claim-admin"}
          onPress={() => handlers.onClaim?.(action.workspaceId)}
        />
      );
  }
}
