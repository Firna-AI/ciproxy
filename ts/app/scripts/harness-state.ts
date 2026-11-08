import type { Problem, ProblemCode, Success } from "../src/api/schema.ts";
import {
  applyPagesControl,
  parsePagesControl,
} from "./harness-pages-controls.ts";
import { releasePages } from "./harness-pages-state.ts";
import type { CapturedBody } from "./harness-pages-facts.ts";
import {
  scenarioName,
  recordedScenario,
  type Scenario,
} from "./harness-scenarios.ts";

export type RequestFact = {
  method: string;
  path: string;
  status: number;
  kind: "api" | "document" | "navigation";
  query?: Record<string, string[]>;
  body?: CapturedBody;
};

export function problem(code: ProblemCode, message: string): Problem {
  return { code, message } satisfies Problem;
}

export const success = { ok: true } satisfies Success;

export type HarnessState = {
  scenario: Scenario;
  build: string;
  facts: RequestFact[];
  issued: number;
  revoked: number;
  sessionReads: number;
};

export function createHarnessState(): HarnessState {
  return {
    scenario: recordedScenario("signed-out"),
    build: "fixture-build-a",
    facts: [],
    issued: 0,
    revoked: 0,
    sessionReads: 0,
  };
}

export function fields(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Local harness controls are separate from the product's generated API. */
export function updateHarness(state: HarnessState, value: unknown): boolean {
  if (!fields(value)) return false;
  const pages =
    value.pages === undefined ? undefined : parsePagesControl(value.pages);
  if (value.pages !== undefined && pages === undefined) return false;
  if (value.scenario !== undefined) {
    const name = scenarioName(value.scenario);
    if (name === undefined) return false;
    releasePages(state.scenario.pages);
    state.scenario = recordedScenario(name);
    if (value.reset !== false) {
      state.facts = [];
      state.issued = 0;
      state.revoked = 0;
      state.sessionReads = 0;
      state.build = "fixture-build-a";
    }
  }
  if (pages !== undefined) {
    applyPagesControl(state.scenario.pages, pages);
    const member = state.scenario.session.workspaces[0];
    if (pages.role !== undefined && member !== undefined)
      member.role = pages.role;
  }
  if (
    typeof value.build === "string" &&
    /^fixture-build-[a-z]+$/.test(value.build)
  )
    state.build = value.build;
  return true;
}
