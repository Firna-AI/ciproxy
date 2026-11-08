// Run from the repository root after app:export. No live API is contacted.
import {
  createServer,
  type IncomingMessage,
  type ServerResponse,
} from "node:http";
import { readFile, stat } from "node:fs/promises";
import { extname, resolve, sep } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { json, readJson, serveApi } from "./harness-api.ts";
import { createHarnessState, updateHarness } from "./harness-state.ts";

const media: Readonly<Record<string, string>> = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript",
  ".css": "text/css",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".ico": "image/x-icon",
  ".woff2": "font/woff2",
  ".ttf": "font/ttf",
  ".map": "application/json",
};

async function staticFile(
  root: string,
  pathname: string,
): Promise<string | undefined> {
  let decoded: string;
  try {
    decoded = decodeURIComponent(pathname);
  } catch {
    return undefined;
  }
  const absolute = resolve(root, `.${decoded}`);
  if (absolute !== root && !absolute.startsWith(root + sep)) return undefined;
  const candidates =
    extname(absolute) === ""
      ? [`${absolute}.html`, resolve(absolute, "index.html")]
      : [absolute];
  for (const candidate of candidates) {
    try {
      if ((await stat(candidate)).isFile()) return candidate;
    } catch {
      /* Next static-export form. */
    }
  }
  return undefined;
}

export async function startHarness(
  port: number,
  root = fileURLToPath(new URL("../dist", import.meta.url)),
  handleRequest?: (
    request: IncomingMessage,
    response: ServerResponse,
    url: URL,
  ) => Promise<boolean>,
) {
  await stat(resolve(root, "index.html"));
  const state = createHarnessState();
  const serve = async (request: IncomingMessage, response: ServerResponse) => {
    const url = new URL(request.url ?? "/", "http://127.0.0.1");
    if (handleRequest && (await handleRequest(request, response, url))) return;
    if (url.pathname === "/__fixture/control" && request.method === "POST") {
      const valid = updateHarness(state, await readJson(request));
      json(response, valid ? 200 : 400, { ok: valid }, state.build);
      return;
    }
    if (url.pathname === "/__fixture/state") {
      json(
        response,
        200,
        {
          scenario: state.scenario.name,
          build: state.build,
          issued: state.issued,
          revoked: state.revoked,
          facts: state.facts,
        },
        state.build,
      );
      return;
    }
    if (url.pathname.startsWith("/api/")) {
      await serveApi(request, response, url.pathname, state);
      return;
    }
    if (url.pathname.startsWith("/__fixture/")) {
      state.facts.push({
        method: request.method ?? "GET",
        path: url.pathname,
        status: 200,
        kind: "navigation",
      });
      response.writeHead(200, {
        "Content-Type": "text/html",
        "Cache-Control": "no-store",
      });
      response.end(
        '<!doctype html><title>Fixture navigation</title><link rel="icon" href="data:,"><h1>Fixture navigation complete</h1>',
      );
      return;
    }
    const file = await staticFile(root, url.pathname);
    if (file === undefined) {
      response.writeHead(404);
      response.end("Static route not exported");
      return;
    }
    const document = extname(file) === ".html";
    if (document)
      state.facts.push({
        method: request.method ?? "GET",
        path: url.pathname,
        status: 200,
        kind: "document",
      });
    response.writeHead(200, {
      "Content-Type": media[extname(file)] ?? "application/octet-stream",
      "Cache-Control": "no-store",
    });
    response.end(await readFile(file));
  };
  const server = createServer((request, response) => {
    void serve(request, response).catch(() => {
      if (!response.headersSent) response.writeHead(500);
      response.end("Fixture request failed");
    });
  });
  await new Promise<void>((done, fail) => {
    server.once("error", fail);
    server.listen(port, "127.0.0.1", done);
  });
  const address = server.address();
  if (address === null || typeof address === "string")
    throw new Error("Harness has no TCP address");
  return {
    origin: `http://127.0.0.1:${address.port}`,
    stop: () =>
      new Promise<void>((done, fail) => {
        server.close((error) => (error ? fail(error) : done()));
      }),
  };
}

if (
  process.argv[1] !== undefined &&
  import.meta.url === pathToFileURL(resolve(process.argv[1])).href
) {
  const argument =
    process.argv.find((value) => value.startsWith("--port="))?.slice(7) ??
    "4340";
  const port = Number(argument);
  if (!Number.isInteger(port) || port < 0 || port > 65535)
    throw new Error("Invalid fixture port");
  const harness = await startHarness(port);
  process.stdout.write(`Fixture dashboard: ${harness.origin}\n`);
  for (const signal of ["SIGINT", "SIGTERM"] as const) {
    process.once(signal, () => {
      void harness.stop().then(() => process.exit(0));
    });
  }
}
