// The signed-out frame: the brand above one centred card on desktop, and a
// full-width column on mobile.

import type { ReactNode } from "react";
import type { Viewport } from "@mokly/mokly";

import { Brand } from "../../components/brand.tsx";
import "./auth.css";

export function AuthShell({
  viewport,
  children,
  footer,
}: {
  viewport: Viewport;
  children: ReactNode;
  footer?: ReactNode;
}) {
  return (
    <div className={`auth auth-${viewport}`}>
      <div className="auth-brand">
        <Brand />
      </div>
      <div className="auth-card">{children}</div>
      {footer ? <div className="auth-footer caption">{footer}</div> : null}
    </div>
  );
}
