import type { paths } from "../../api/index.ts";
import type { Transport } from "../src/api/client.ts";
import { object, string } from "../src/api/parse-core.ts";
import { parseOverflowSettings } from "../src/api/parse-overflow.ts";
import { parseSession } from "../src/api/parse-session.ts";

const sessionPath = "/session" satisfies keyof paths;
const settingsPath =
  "/workspaces/{workspace_id}/overflow" satisfies keyof paths;

type CheckStage =
  | "private credential file"
  | "current Admin session and CSRF"
  | "typed Boat settings and API build";

type Credentials = {
  base_url: string;
  workspace_id: string;
  session_cookie: string;
  csrf_token: string;
};

export type BoatLiveDependencies = {
  readCredentials: (path: string) => Promise<string>;
  transport: Transport;
  write: (text: string) => void;
};

function credentials(value: unknown): Credentials {
  const fields = object(value, "private credential file");
  const base = new URL(string(fields.base_url));
  const workspace_id = string(fields.workspace_id);
  const session_cookie = string(fields.session_cookie);
  const csrf_token = string(fields.csrf_token);
  if (
    base.protocol !== "https:" ||
    base.username !== "" ||
    base.password !== "" ||
    base.pathname !== "/" ||
    base.search !== "" ||
    base.hash !== "" ||
    session_cookie.includes(";") ||
    [workspace_id, session_cookie, csrf_token].some(
      (value) => value === "" || /[\r\n]/.test(value),
    )
  ) {
    throw new Error("Invalid private credential file");
  }
  return { base_url: base.origin, workspace_id, session_cookie, csrf_token };
}

const help = `Usage: node ts/app/scripts/boat-live-smoke.ts [check|--help]

The default command is check. It reads the session and Overflow settings.
Set CIPROXY_BOAT_CREDENTIALS_FILE to a private JSON file outside the repository.
Required strings: base_url, workspace_id, session_cookie, csrf_token.
Use an Admin session and its current CSRF token from GET /api/session.
The base_url must be an HTTPS origin with no path.
The session_cookie is the value of the __Host-ciproxy cookie.

This command prints fixed check names and results. It prints no private values.
It makes no change to an account, key, policy, GitHub job, or Boat sandbox.
It does not check live connect, key change, policy writes, or disconnect.
Exit status: 0 pass, 1 fail, 2 skip.
`;

export async function runBoatLiveCheck(
  args: readonly string[],
  credentialFile: string | undefined,
  dependencies: BoatLiveDependencies,
): Promise<0 | 1 | 2> {
  const { readCredentials, transport, write } = dependencies;
  if (args.length === 1 && (args[0] === "--help" || args[0] === "-h")) {
    write(help);
    return 0;
  }
  if (args.length > 1 || (args.length === 1 && args[0] !== "check")) {
    write("FAIL live Boat command. Use --help.\n");
    return 1;
  }
  if (!credentialFile) {
    write("SKIP live Boat: CIPROXY_BOAT_CREDENTIALS_FILE is absent\n");
    return 2;
  }

  let stage: CheckStage = "private credential file";
  let passed = 0;
  const pass = () => {
    passed += 1;
    write(`PASS ${stage}\n`);
  };
  try {
    const value: unknown = JSON.parse(await readCredentials(credentialFile));
    const config = credentials(value);
    pass();
    let build: string | undefined;

    async function read<T>(
      path: string,
      parse: (value: unknown) => T | undefined,
    ) {
      const response = await transport(config.base_url + "/api" + path, {
        method: "GET",
        headers: {
          Accept: "application/json",
          Cookie: `__Host-ciproxy=${config.session_cookie}`,
          Origin: config.base_url,
          "x-ciproxy-csrf": config.csrf_token,
        },
        redirect: "error",
        cache: "no-store",
      });
      const currentBuild = response.headers.get("x-ciproxy-build");
      if (
        response.status !== 200 ||
        !currentBuild ||
        (build !== undefined && build !== currentBuild)
      ) {
        throw new Error("Live Boat API check failed");
      }
      build = currentBuild;
      const data: unknown = await response.json();
      const parsed = parse(data);
      if (parsed === undefined) throw new Error("Invalid live Boat response");
      return parsed;
    }

    stage = "current Admin session and CSRF";
    const session = await read(sessionPath, parseSession);
    if (
      session.state !== "ready" ||
      session.csrf_token !== config.csrf_token ||
      !session.workspaces.some(
        (workspace) =>
          workspace.id === config.workspace_id && workspace.role === "admin",
      )
    ) {
      throw new Error("Live Boat session check failed");
    }
    pass();

    stage = "typed Boat settings and API build";
    await read(
      settingsPath.replace(
        "{workspace_id}",
        encodeURIComponent(config.workspace_id),
      ),
      parseOverflowSettings,
    );
    pass();
    write(`Live Boat checks ${passed}, pass ${passed}, fail 0, skip 0\n`);
    return 0;
  } catch {
    write(`FAIL ${stage}\n`);
    write(`Live Boat checks ${passed + 1}, pass ${passed}, fail 1, skip 0\n`);
    return 1;
  }
}
