import type { components } from "../../api/index.ts";
import { jobsModes, type JobsMode } from "./fixtures-jobs-live.ts";
import {
  savingsModes,
  savingsRanges,
  type SavingsMode,
} from "./fixtures-savings.ts";
import {
  usageFixture,
  usageModes,
  type UsageMode,
} from "./fixtures-billing-usage.ts";
import {
  paymentStates,
  type PaymentState,
} from "./fixtures-billing-payment.ts";
import {
  pageRoutes,
  releasePages,
  settlePayment,
  type PageRoute,
  type PagesFixtureState,
} from "./harness-pages-state.ts";

export type PagesControl = {
  jobs?: JobsMode;
  history?: "populated" | "empty";
  detail?: "found" | "not-found";
  savings?: SavingsMode;
  range?: components["schemas"]["SavingsRange"];
  usage?: UsageMode;
  payment?: PaymentState;
  returned?: PaymentState;
  role?: components["schemas"]["WorkspaceRole"] | null;
  inactiveOrganizations?: boolean;
  failures?: PageRoute[];
  held?: PageRoute[];
};

function oneOf<T extends string>(
  value: unknown,
  values: readonly T[],
): T | undefined {
  return values.find((item) => item === value);
}

export function parsePagesControl(value: unknown): PagesControl | undefined {
  if (typeof value !== "object" || value === null || Array.isArray(value))
    return undefined;
  const control: PagesControl = {};
  for (const [key, raw] of Object.entries(value)) {
    switch (key) {
      case "jobs": {
        const parsed = oneOf(raw, jobsModes);
        if (parsed === undefined) return undefined;
        control.jobs = parsed;
        break;
      }
      case "history": {
        const parsed = oneOf(raw, ["populated", "empty"] as const);
        if (parsed === undefined) return undefined;
        control.history = parsed;
        break;
      }
      case "detail": {
        const parsed = oneOf(raw, ["found", "not-found"] as const);
        if (parsed === undefined) return undefined;
        control.detail = parsed;
        break;
      }
      case "savings": {
        const parsed = oneOf(raw, savingsModes);
        if (parsed === undefined) return undefined;
        control.savings = parsed;
        break;
      }
      case "range": {
        const parsed = oneOf(raw, savingsRanges);
        if (parsed === undefined) return undefined;
        control.range = parsed;
        break;
      }
      case "usage": {
        const parsed = oneOf(raw, usageModes);
        if (parsed === undefined) return undefined;
        control.usage = parsed;
        break;
      }
      case "payment":
      case "returned": {
        const parsed = oneOf(raw, paymentStates);
        if (parsed === undefined) return undefined;
        control[key] = parsed;
        break;
      }
      case "role": {
        const parsed =
          raw === null
            ? null
            : oneOf(raw, ["admin", "billing", "viewer"] as const);
        if (parsed === undefined) return undefined;
        control.role = parsed;
        break;
      }
      case "inactiveOrganizations":
        if (typeof raw !== "boolean") return undefined;
        control.inactiveOrganizations = raw;
        break;
      case "failures":
      case "held": {
        if (!Array.isArray(raw)) return undefined;
        const routes: PageRoute[] = [];
        for (const item of raw) {
          const parsed = oneOf(item, pageRoutes);
          if (parsed === undefined) return undefined;
          routes.push(parsed);
        }
        control[key] = routes;
        break;
      }
      default:
        return undefined;
    }
  }
  return control;
}

export function applyPagesControl(
  state: PagesFixtureState,
  control: PagesControl,
): void {
  if (control.jobs !== undefined) state.jobs = control.jobs;
  if (control.history !== undefined) state.history = control.history;
  if (control.detail !== undefined) state.detail = control.detail;
  if (control.savings !== undefined) state.savings = control.savings;
  if (control.range !== undefined) state.range = control.range;
  if (control.usage !== undefined) {
    state.usage = usageFixture(control.usage);
    state.payment =
      state.usage.status.selection.tier === "free" ? "open" : "confirmed";
  }
  if (control.payment !== undefined) state.payment = control.payment;
  if (control.returned !== undefined) {
    state.returned = control.returned;
    settlePayment(state, control.returned);
  }
  if (control.inactiveOrganizations !== undefined)
    state.inactiveOrganizations = control.inactiveOrganizations;
  if (control.failures !== undefined) state.failures = control.failures;
  if (control.held !== undefined) releasePages(state, control.held);
}
