import { once } from "node:events";
import { readFile } from "node:fs/promises";
import { createServer, type ServerResponse } from "node:http";
import { extname, join, resolve, sep } from "node:path";

import { buildSite } from "../built-site.ts";

const contentTypes: Readonly<Record<string, string>> = {
  ".css": "text/css",
  ".html": "text/html",
  ".js": "text/javascript",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".wasm": "application/wasm",
  ".woff2": "font/woff2",
};

/** Serve a fresh production build, with the real search index. */
export async function serveSite() {
  const site = buildSite();

  async function respond(url: string, response: ServerResponse): Promise<void> {
    try {
      const pathname = decodeURIComponent(
        new URL(url, "http://localhost").pathname,
      );
      const path = resolve(site.directory, `.${pathname}`);
      if (
        !path.startsWith(`${site.directory}${sep}`) &&
        path !== site.directory
      ) {
        response.writeHead(404).end();
        return;
      }
      const file = pathname.endsWith("/") ? join(path, "index.html") : path;
      const body = await readFile(file);
      response.writeHead(200, {
        "Content-Type":
          contentTypes[extname(file)] ?? "application/octet-stream",
      });
      response.end(body);
    } catch {
      response.writeHead(404).end();
    }
  }

  const server = createServer((request, response) => {
    void respond(request.url ?? "/", response);
  });
  try {
    server.listen(0, "127.0.0.1");
    await once(server, "listening");
    const address = server.address();
    if (!address || typeof address === "string")
      throw new Error("No site port");
    return {
      origin: `http://127.0.0.1:${address.port}`,
      close: async () => {
        try {
          await new Promise<void>((accept, reject) => {
            server.close((error) => (error ? reject(error) : accept()));
          });
        } finally {
          site.remove();
        }
      },
    };
  } catch (error) {
    server.close();
    site.remove();
    throw error;
  }
}
