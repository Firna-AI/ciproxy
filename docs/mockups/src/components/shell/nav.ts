// The main navigation of the dashboard, in the order of the sidebar and the
// mobile tab bar. Settings is for Admins only.

import type { LucideIcon } from "lucide-react";
import { Gauge, ListChecks, PiggyBank, Server, Settings } from "lucide-react";

import type { Role } from "../../fixtures/members.ts";
import { paths, type CataloguePath } from "../paths.ts";

export type NavKey = "jobs" | "machines" | "savings" | "billing" | "settings";

export type NavItem = {
  key: NavKey;
  label: string;
  mobileLabel: string;
  to: CataloguePath;
  icon: LucideIcon;
};

export const navItems: readonly NavItem[] = [
  {
    key: "jobs",
    label: "Jobs",
    mobileLabel: "Jobs",
    to: paths.jobs,
    icon: ListChecks,
  },
  {
    key: "machines",
    label: "Machines",
    mobileLabel: "Machines",
    to: paths.machines,
    icon: Server,
  },
  {
    key: "savings",
    label: "Savings",
    mobileLabel: "Savings",
    to: paths.savings,
    icon: PiggyBank,
  },
  {
    key: "billing",
    label: "Plan and usage",
    mobileLabel: "Plan",
    to: paths.billing,
    icon: Gauge,
  },
  {
    key: "settings",
    label: "Settings",
    mobileLabel: "Settings",
    to: paths.settings,
    icon: Settings,
  },
];

/** The items that a person with this role sees. */
export function navFor(role: Role): readonly NavItem[] {
  return role === "admin"
    ? navItems
    : navItems.filter((item) => item.key !== "settings");
}
