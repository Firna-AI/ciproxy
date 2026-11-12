import { resolve } from "node:path";

import { selectSmokeRuns } from "./browser-registry.ts";
import { createPagesRegistry } from "./browser-pages-registry.ts";
import { runPagesSmoke } from "./browser-pages-runtime.ts";

async function main(): Promise<void> {
  const args = process.argv.slice(2);
  if (args.includes("--help")) {
    process.stdout.write(
      "M21 page smoke: --case=NAME[,NAME] --repeat=COUNT --width=WIDTH[,WIDTH] --stop-on-failure --list\n",
    );
    return;
  }
  const origin = process.env.CIPROXY_HARNESS_URL ?? "http://127.0.0.1:4340";
  const output = resolve(
    process.env.CIPROXY_SMOKE_OUTPUT ?? ".context/m21/browser-smoke",
  );
  const registry = createPagesRegistry(origin, output);
  if (args.includes("--list")) {
    registry.forEach((entry) => process.stdout.write(`${entry.name}\n`));
    process.stdout.write(`Page cases ${registry.length}\n`);
    if (registry.length === 0) process.exitCode = 2;
    return;
  }
  const repeat = Number(
    args.find((value) => value.startsWith("--repeat="))?.slice(9) ?? "1",
  );
  if (!Number.isInteger(repeat) || repeat < 1)
    throw new Error("The repeat count must be a positive integer.");
  const selected = selectSmokeRuns(registry, args, (message) =>
    process.stderr.write(`WARNING ${message}\n`),
  );
  if (selected.length === 0)
    throw new Error("Page smoke cases need the product interface notes.");
  const results = await runPagesSmoke(
    (directory) => createPagesRegistry(origin, directory),
    new Set(selected.map((entry) => entry.name)),
    output,
    repeat,
    {
      widths: args.some((value) => value.startsWith("--width="))
        ? (
            args
              .find((value) => value.startsWith("--width="))
              ?.slice(8)
              .split(",")
              .map(Number) ?? []
          ).map((width) => {
            if (![1440, 720, 390].includes(width))
              throw new Error("Select a required smoke width.");
            return width;
          })
        : undefined,
      stopOnFailure: args.includes("--stop-on-failure"),
    },
  );
  const passed = results.filter((result) => result.status === "pass").length;
  process.stdout.write(
    `Checks ${results.length}; pass ${passed}; fail ${results.length - passed}; skip 0\n`,
  );
  if (results.length === 0 || passed !== results.length) process.exitCode = 1;
}

try {
  await main();
} catch (error) {
  process.stderr.write(
    `FAIL ${error instanceof Error ? error.message : "Page smoke setup"}\n`,
  );
  process.exitCode = 1;
}
