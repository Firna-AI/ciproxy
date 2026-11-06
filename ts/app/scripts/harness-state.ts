import type { Problem, ProblemCode, Success } from "../src/api/schema.ts";
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
};

export function createHarnessState(): HarnessState {
  return {
    scenario: recordedScenario("signed-out"),
    build: "fixture-build-a",
    facts: [],
    issued: 0,
    revoked: 0,
  };
}

export function fields(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

/** Local harness controls are separate from the product's generated API. */
export function updateHarness(state: HarnessState, value: unknown): boolean {
  if (!fields(value)) return false;
  if (value.scenario !== undefined) {
    const name = scenarioName(value.scenario);
    if (name === undefined) return false;
    state.scenario = recordedScenario(name);
    if (value.reset !== false) {
      state.facts = [];
      state.issued = 0;
      state.revoked = 0;
      state.build = "fixture-build-a";
    }
  }
  if (
    typeof value.build === "string" &&
    /^fixture-build-[a-z]+$/.test(value.build)
  )
    state.build = value.build;
  return true;
}
