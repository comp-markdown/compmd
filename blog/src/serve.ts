// `cblo build` and `cblo dev` (docs/blog-tool.md, decision 3): a built blog written into a folder, or served, built
// again for each page asked for, so a saved post shows when the page is reloaded.

import { mkdir, rm, writeFile } from "node:fs/promises";
import { createServer, type Server } from "node:http";
import { dirname, extname, join } from "node:path";
import { readConfig } from "./config.ts";
import { buildSite, type BuildOptions, type Site } from "./build.ts";

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
};

/** The file of `site` a request's path asks for, under the blog's own path `base`: a folder's `index.html`. */
export function fileFor(site: Site, base: string, pathname: string): string | undefined {
  if (!pathname.startsWith(base)) return undefined;
  const path = decodeURIComponent(pathname.slice(base.length));
  const file = path === "" || path.endsWith("/") ? `${path}index.html` : path;
  return site.files.has(file) ? file : undefined;
}

/**
 * Serves the blog in `dir` on `port`, at its address's path (`/blog/`), on this machine only: each request builds it
 * again, so a saved post shows when its page is reloaded, and an error in it shows on the page.
 */
export async function serve(dir: string, port: number, options: BuildOptions): Promise<Server> {
  const base = new URL((await readConfig(dir)).url).pathname;
  const server = createServer((request, response) => {
    const pathname = new URL(request.url ?? "/", "http://localhost").pathname;
    buildSite(dir, options).then(
      (site) => {
        const file = fileFor(site, base, pathname);
        if (file === undefined) {
          response.writeHead(pathname === "/" ? 302 : 404, pathname === "/" ? { location: base } : { "content-type": "text/plain; charset=utf-8" });
          response.end(pathname === "/" ? "" : `Not in the blog: ${pathname}`);
          return;
        }
        response.writeHead(200, { "content-type": TYPES[extname(file)] ?? "application/octet-stream" });
        response.end(site.files.get(file));
      },
      (error: unknown) => {
        response.writeHead(500, { "content-type": "text/plain; charset=utf-8" });
        response.end(error instanceof Error ? error.message : String(error));
      },
    );
  });
  await new Promise<void>((done) => server.listen(port, "127.0.0.1", done));
  return server;
}
