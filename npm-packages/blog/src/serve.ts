// `cblo build` and `cblo dev` (docs/blog-tool.md, decision 3): a built blog written into a folder, or served, built
// again for each page asked for, its open pages reloaded when it is saved.

import { mkdir, rm, writeFile } from "node:fs/promises";
import { Server, type RequestListener, type ServerResponse } from "node:http";
import { dirname, extname, join } from "node:path";
import { CONTRACT_HEADERS } from "@compmd/vite";
import { blogSources } from "./blog-change.ts";
import { watchBlog } from "./blog-watch.ts";
import { ClientCache } from "./client-cache.ts";
import { readConfig } from "./config.ts";
import { buildSite, type BuildOptions, type Site } from "./build.ts";
import { escape } from "./html.ts";
import { RELOAD_PATH, ReloadEvents, withReload } from "./reload-events.ts";

/** Writes the blog in `dir`, built, into `out`, emptied first. Its warnings. */
export async function writeSite(dir: string, out: string, options: BuildOptions): Promise<string[]> {
  const { files, warnings } = await buildSite(dir, options);
  await rm(out, { recursive: true, force: true });
  for (const [path, content] of files) {
    const file = join(out, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, content);
  }
  return warnings;
}

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".xml": "application/xml; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".jpg": "image/jpeg",
  ".jpeg": "image/jpeg",
  ".gif": "image/gif",
  ".webp": "image/webp",
  ".csv": "text/csv; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
};

/** The file of `site` a request's path asks for, under the blog's own path `base`: a folder's `index.html`. */
export function fileFor(site: Site, base: string, pathname: string): string | undefined {
  if (!pathname.startsWith(base)) return undefined;
  const path = decodeURIComponent(pathname.slice(base.length));
  const file = path === "" || path.endsWith("/") ? `${path}index.html` : path;
  return site.files.has(file) ? file : undefined;
}

/**
 * Sends `file` of `site` as the response, with its content type, and the sandbox's frames and packs with the headers
 * every host serves them with (docs/hosting.md, decision 8).
 */
export function sendBlogFile(response: ServerResponse, site: Site, file: string): void {
  const contract = file.startsWith("compmd/frames/") || file.startsWith("compmd/packs/") ? CONTRACT_HEADERS : {};
  response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream", ...contract });
  response.end(site.files.get(file));
}

/** `cblo dev`'s server, which stops its watch and its pages' streams when it closes, so it can close. */
class DevServer extends Server {
  readonly #stop: () => void;

  constructor(listener: RequestListener, stop: () => void) {
    super(listener);
    this.#stop = stop;
  }

  override close(callback?: (error?: Error) => void): this {
    this.#stop();
    return super.close(callback);
  }
}

/**
 * Serves the blog in `dir` on `port`, at its address's path (`/blog/`), on this machine only: each request builds it
 * again, and an error in it shows on the page. Its pages reload when the blog's files are saved, and the page's script
 * is built again when what it is made from changes.
 */
export async function serve(dir: string, port: number, options: BuildOptions): Promise<Server> {
  const base = new URL((await readConfig(dir)).url).pathname;
  const clients = new ClientCache();
  const events = new ReloadEvents();
  const watched = watchBlog(await blogSources(dir), (change) => {
    if (change === "client") clients.forget();
    events.reload();
  });
  const server = new DevServer(
    (request, response) => {
      const pathname = new URL(request.url ?? "/", "http://localhost").pathname;
      if (pathname === RELOAD_PATH) {
        events.open(response);
        return;
      }
      buildSite(dir, { ...options, clients }).then(
        (site) => {
          const file = fileFor(site, base, pathname);
          if (file === undefined) {
            response.writeHead(pathname === "/" ? 302 : 404, pathname === "/" ? { location: base } : { "content-type": "text/plain; charset=utf-8" });
            response.end(pathname === "/" ? "" : `Not in the blog: ${pathname}`);
            return;
          }
          const content = site.files.get(file);
          const page = typeof content === "string" && file.endsWith(".html") ? new Map([[file, withReload(content)]]) : site.files;
          sendBlogFile(response, { ...site, files: page }, file);
        },
        (error: unknown) => {
          response.writeHead(500, { "content-type": "text/html; charset=utf-8" });
          response.end(withReload(`<pre>${escape(error instanceof Error ? error.message : String(error))}</pre>`));
        },
      );
    },
    () => {
      watched.close();
      events.close();
    },
  );
  await new Promise<void>((done) => server.listen(port, "127.0.0.1", done));
  return server;
}
