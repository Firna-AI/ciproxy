import type { ApiResult } from "../src/api/client.ts";

export type LiveCheck = <T>(
  name: string,
  run: () => T | Promise<T>,
) => Promise<T>;

export function apiValue<T>(result: ApiResult<T>): T {
  if (!result.ok) throw new Error("The live API request failed.");
  return result.value;
}

export function createLiveChecks() {
  const results: {
    name: string;
    status: "pass" | "fail" | "skip" | "pending";
  }[] = [];
  const check: LiveCheck = async (name, run) => {
    try {
      const value = await run();
      results.push({ name, status: "pass" });
      process.stdout.write(`PASS ${name}\n`);
      return value;
    } catch {
      results.push({ name, status: "fail" });
      process.stderr.write(`FAIL ${name}\n`);
      throw new Error("A live check failed.");
    }
  };
  return {
    check,
    fail(name: string) {
      results.push({ name, status: "fail" });
      process.stderr.write(`FAIL ${name}\n`);
    },
    skip(name: string) {
      results.push({ name, status: "skip" });
      process.stdout.write(`SKIP ${name}\n`);
    },
    pending(name: string) {
      results.push({ name, status: "pending" });
      process.stdout.write(`PENDING ${name}\n`);
    },
    failed: () => results.some((result) => result.status === "fail"),
    exitCode: () =>
      results.some((result) => result.status === "fail")
        ? 1
        : results.some(
              (result) =>
                result.status === "pending" || result.status === "skip",
            )
          ? 2
          : 0,
    summary() {
      const count = (status: "pass" | "fail" | "skip" | "pending") =>
        results.filter((result) => result.status === status).length;
      process.stdout.write(
        `Live checks ${results.length}; pass ${count("pass")}; fail ${count("fail")}; skip ${count("skip")}; pending ${count("pending")}\n`,
      );
    },
  };
}

export type LiveChecks = ReturnType<typeof createLiveChecks>;
