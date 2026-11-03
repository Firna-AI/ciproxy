// Stripe Checkout as Stripe hosts it, with the line items that ciproxy sends.
// Stripe owns this page; the mockup shows what the customer sees in the flow.

import { ArrowLeft, Lock } from "lucide-react";
import { MockLink, type Viewport } from "@mokly/mokly";

import { LinkAction } from "../../components/actions.tsx";
import { BrandMark } from "../../components/brand.tsx";
import { paths } from "../../components/paths.ts";
import { formatMoneyCents } from "../../fixtures/format.ts";
import { tiers } from "../../fixtures/tiers.ts";
import "./stripe.css";

function Field({
  label,
  value,
  placeholder,
}: {
  label: string;
  value?: string;
  placeholder?: string;
}) {
  return (
    <label className="stripe-field">
      <span className="stripe-label">{label}</span>
      <span className={`stripe-input${value ? "" : " is-empty"}`}>
        {value ?? placeholder}
      </span>
    </label>
  );
}

export function StripeCheckoutScreen({ viewport }: { viewport: Viewport }) {
  const price = formatMoneyCents(tiers.team.price);
  return (
    <div className={`stripe stripe-${viewport}`}>
      <section className="stripe-summary">
        <MockLink to={paths.checkoutCancelled} className="stripe-merchant">
          <ArrowLeft size={16} />
          <BrandMark size={22} />
          <span>ciproxy</span>
        </MockLink>
        <p className="stripe-muted">Subscribe to Team</p>
        <p className="stripe-amount">
          {price} <span className="stripe-muted">per month</span>
        </p>
        <div className="stripe-lines">
          <div className="stripe-line">
            <span>
              Team
              <span className="stripe-muted stripe-sub">
                25,000 starts and 3 own nodes · billed monthly
              </span>
            </span>
            <span>{price}</span>
          </div>
          <div className="stripe-line stripe-total">
            <span>Total due today</span>
            <span>{price}</span>
          </div>
        </div>
      </section>
      <section className="stripe-form">
        <h1 className="stripe-heading">Pay with card</h1>
        <Field label="Email" value="cal@mokly.ai" />
        <div className="stripe-field">
          <span className="stripe-label">Card information</span>
          <span className="stripe-card">
            <span className="stripe-input is-empty">1234 1234 1234 1234</span>
            <span className="stripe-card-row">
              <span className="stripe-input is-empty">MM / YY</span>
              <span className="stripe-input is-empty">CVC</span>
            </span>
          </span>
        </div>
        <Field label="Cardholder name" placeholder="Full name on card" />
        <Field label="Country or region" value="United Kingdom" />
        <LinkAction
          to={paths.checkoutConfirming}
          tone="primary"
          size="lg"
          icon={Lock}
          block
        >
          Subscribe
        </LinkAction>
        <p className="stripe-foot">Powered by stripe · Terms · Privacy</p>
      </section>
    </div>
  );
}
