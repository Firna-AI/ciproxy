import type { Page } from "./browser-driver.ts";
import { control, visible } from "./browser-fixture.ts";
import type { PagesControl } from "./harness-pages-controls.ts";
import type { ScenarioName } from "./harness-scenarios.ts";
import type { PagesSmokeRun } from "./browser-pages-runtime.ts";
import { apiAction } from "./browser-pages-helpers.ts";

export type PageScreen = {
  name: string;
  scenario: ScenarioName;
  path: string;
  heading?: string;
  text?: string | RegExp;
  role?: { role: string; name: string | RegExp };
  pages?: PagesControl;
  statuses?: readonly number[];
  check?: (page: Page) => Promise<void>;
  layoutCheck?: (page: Page, width: number) => Promise<void>;
  init?: string;
  response?: { path: string; method: string };
};

export function screenRuns(
  origin: string,
  screens: readonly PageScreen[],
): PagesSmokeRun[] {
  return screens.map((screen) => ({
    name: screen.name,
    expectedHttpStatuses: screen.statuses,
    init: screen.init,
    async check(page, _context, width) {
      await control(origin, { scenario: screen.scenario, pages: screen.pages });
      const open = async () => {
        await page.goto(origin + screen.path, {
          waitUntil: "domcontentloaded",
        });
      };
      if (screen.response !== undefined)
        await apiAction(
          page,
          screen.response.path,
          screen.response.method,
          open,
        );
      else await open();
      if (screen.heading !== undefined)
        await visible(
          page.getByRole("heading", { name: screen.heading, exact: true }),
        );
      if (screen.text !== undefined)
        await visible(
          page.getByText(screen.text, {
            exact: typeof screen.text === "string",
          }),
        );
      if (screen.role !== undefined)
        await visible(
          page.getByRole(screen.role.role, {
            name: screen.role.name,
            exact: true,
          }),
        );
      await screen.check?.(page);
      await screen.layoutCheck?.(page, width);
    },
  }));
}
