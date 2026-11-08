import { readFile } from "node:fs/promises";

import { runBoatLiveCheck } from "./boat-live-check.ts";

process.exitCode = await runBoatLiveCheck(
  process.argv.slice(2),
  process.env.CIPROXY_BOAT_CREDENTIALS_FILE,
  {
    readCredentials: (path) => readFile(path, "utf8"),
    transport: (url, init) =>
      fetch(url, { ...init, signal: AbortSignal.timeout(15_000) }),
    write: (text) => {
      process.stdout.write(text);
    },
  },
);
