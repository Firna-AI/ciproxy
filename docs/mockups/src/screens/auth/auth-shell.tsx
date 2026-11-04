// The signed-out frame: the brand above one centred card on desktop, and a
// full-width column on mobile. On sign-in, the brand leads back to the site.

import type { ReactNode } from "react";
import { MockLink, type Viewport } from "@mokly/mokly";

import { Brand } from "../../components/brand.tsx";
import type { CataloguePath } from "../../components/paths.ts";
import "./auth.css";

export function AuthShell({
  viewport,
  children,
  footer,
  brandTo,
}: {
  viewport: Viewport;
  children: ReactNode;
  footer?: ReactNode;
  /** The page that the brand opens, such as the landing page. */
  brandTo?: CataloguePath;
}) {
  return (
    <div className={`auth auth-${viewport}`}>
      <div className="auth-brand">
        {brandTo ? (
          <MockLink to={brandTo} className="auth-home">
            <Brand />
          </MockLink>
        ) : (
          <Brand />
        )}
      </div>
      <div className="auth-card">{children}</div>
      {footer ? <div className="auth-footer caption">{footer}</div> : null}
    </div>
  );
}
