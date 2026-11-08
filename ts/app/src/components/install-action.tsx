// Keep the install control's link styles while saving its entry before GitHub.

import { useRef, useState } from "react";

import { useServices } from "../services/context.tsx";
import { installFailure } from "../view/install.ts";
import type { PageAction } from "../view/page-model.ts";
import { ActionLink, type Size } from "./actions.tsx";
import { Notice } from "./notice.tsx";

export function InstallAction({
  action,
  size,
  block,
}: {
  action: Extract<PageAction, { kind: "install" }>;
  size: Size;
  block: boolean;
}) {
  const services = useServices();
  const running = useRef(false);
  const [failed, setFailed] = useState(false);
  return (
    <>
      {failed ? (
        <Notice tone="danger" title={installFailure.title}>
          {installFailure.body}
        </Notice>
      ) : null}
      <ActionLink
        label={action.label}
        href={action.href}
        tone={action.tone}
        size={size}
        block={block}
        onClick={(event) => {
          event.preventDefault();
          if (running.current) return;
          running.current = true;
          setFailed(false);
          void services.startInstallation().then((result) => {
            running.current = false;
            setFailed(!result.ok);
          });
        }}
      />
    </>
  );
}
