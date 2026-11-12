import { readLivePagesCredentials } from "./live-pages-config.ts";
import { createLiveChecks } from "./live-pages-checks.ts";
import { createLivePagesConnection } from "./live-pages-session.ts";
import { readLivePages } from "./live-pages-reads.ts";
import { startLiveCheckout } from "./live-pages-checkout.ts";
import { checkLiveReturn } from "./live-pages-return.ts";

const checks = createLiveChecks();
try {
  const credentials = await readLivePagesCredentials();
  if (credentials === undefined) {
    process.stderr.write("SKIP live M21 credentials absent\n");
    process.exitCode = 2;
  } else {
    const mode =
      process.argv.find((value) => value.startsWith("--step="))?.slice(7) ??
      "read";
    if (mode !== "read" && mode !== "checkout" && mode !== "return")
      throw new Error("Unknown live step.");
    const connection = createLivePagesConnection(credentials);
    await checks.check("session access and current CSRF", () =>
      connection.session(),
    );
    if (mode === "read")
      await readLivePages(connection.client, credentials.workspace_id, checks);
    if (mode === "checkout")
      await startLiveCheckout(connection.client, credentials, checks.check);
    if (mode === "return")
      await checkLiveReturn(connection.client, credentials, checks);
    await checks.check("API build header on every response", () => {
      if (connection.missingBuild())
        throw new Error("An API build header is absent.");
    });
    checks.summary();
    process.exitCode = checks.exitCode();
  }
} catch {
  if (!checks.failed()) checks.fail("live M21 configuration or saved context");
  checks.summary();
  process.exitCode = 1;
}
