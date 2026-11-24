// `cblo build` and `cblo dev` (docs/blog-tool.md, decision 3): a built blog written into a folder, or served, built
// again for each page asked for, its open pages reloaded when it is saved.

import { mkdir, rm, writeFile } from "node:fs/promises";
import { Server, type RequestListener, type ServerResponse } from "node:http";
import { dirname, join, resolve, sep } from "node:path";
import { CONTRACT_HEADERS } from "@compmd/vite";
import { blogSources } from "./blog-change.ts";
import { watchBlog } from "./blog-watch.ts";
import { ClientCache } from "./client-cache.ts";
import { BlogError, readConfig } from "./config.ts";
import { buildSite, type BuildOptions, type Site } from "./build.ts";
import { escape } from "./html.ts";
import { POSTS } from "./posts.ts";
import { RELOAD_PATH, ReloadEvents, withReload } from "./reload-events.ts";
import { isFromThisMachine, requestedPath, sendBadRequest, sendFile } from "./static-site.ts";

/**
 * Where the blog in `dir` is written: `out` read against the blog's folder, never the blog itself, a folder holding it,
 * or one inside its posts, which emptying it first would delete (audit S24).
 *
 * @throws BlogError naming why `out` can't be written.
 */
export function outputFolder(dir: string, out: string): string {
  const blog = resolve(dir);
  const target = resolve(blog, out);
  const posts = join(blog, POSTS);
  const within = (folder: string, inside: string) => inside === folder || inside.startsWith(folder + sep);
  if (within(target, blog)) throw new BlogError(`${out} is the blog or holds it, and is emptied before the blog is written: write it into a folder of its own, such as dist`);
  if (within(posts, target)) throw new BlogError(`${out} is inside the blog's ${POSTS}/: write it into a folder of its own, such as dist`);
  return target;
}

/** Writes the blog in `dir`, built, into `out` (read against `dir`), emptied first. Its warnings. @throws BlogError as {@link outputFolder} does. */
export async function writeSite(dir: string, out: string, options: BuildOptions): Promise<string[]> {
  const target = outputFolder(dir, out);
  const { files, warnings } = await buildSite(dir, options);
  await rm(target, { recursive: true, force: true });
  for (const [path, content] of files) {
    const file = join(target, ...path.split("/"));
    await mkdir(dirname(file), { recursive: true });
    await writeFile(file, content);
  }
  return warnings;
}

/** The file of `site` a request's path asks for, under the blog's own path `base`: a folder's `index.html`; none for a path that can't be read. */
export function fileFor(site: Site, base: string, pathname: string): string | undefined {
  if (!pathname.startsWith(base)) return undefined;
  let path: string;
  try {
    path = decodeURIComponent(pathname.slice(base.length));
  } catch {
    return undefined;
  }
  const file = path === "" || path.endsWith("/") ? `${path}index.html` : path;
  return site.files.has(file) ? file : undefined;
}

/**
 * Sends `file` of `site` as the response, with its content type and `nosniff`, and the sandbox's frames and packs with
 * the headers every host serves them with (docs/hosting.md, decision 8).
 */
export function sendBlogFile(response: ServerResponse, site: Site, file: string): void {
  const contract = file.startsWith("compmd/frames/") || file.startsWith("compmd/packs/") ? CONTRACT_HEADERS : {};
  sendFile(response, file, site.files.get(file) ?? "", contract);
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
      // It serves drafts: only to this machine's browser, never a page of another site that names it (audit S29).
      if (!isFromThisMachine(request)) {
        sendBadRequest(response, "This server answers only this machine: open it at localhost.");
        return;
      }
      const decoded = requestedPath(request);
      if (decoded === undefined) {
        sendBadRequest(response, "The address can't be read.");
        return;
      }
      const pathname = new URL(request.url ?? "/", "http://localhost").pathname;
      if (pathname === RELOAD_PATH) {
        events.open(response);
        return;
      }
      buildSite(dir, { ...options, clients })
        .then((site) => {
          const file = fileFor(site, base, pathname);
          if (file === undefined) {
            response.writeHead(pathname === "/" ? 302 : 404, pathname === "/" ? { location: base } : { "content-type": "text/plain; charset=utf-8" });
            response.end(pathname === "/" ? "" : `Not in the blog: ${pathname}`);
            return;
          }
          const content = site.files.get(file);
          const page = typeof content === "string" && file.endsWith(".html") ? new Map([[file, withReload(content)]]) : site.files;
          sendBlogFile(response, { ...site, files: page }, file);
        })
        .catch((error: unknown) => {
          if (response.headersSent) {
            response.destroy();
            return;
          }
          response.writeHead(500, { "content-type": "text/html; charset=utf-8" });
          response.end(withReload(`<pre>${escape(error instanceof Error ? error.message : String(error))}</pre>`));
        });
    },
    () => {
      watched.close();
      events.close();
    },
  );
  await new Promise<void>((done) => server.listen(port, "127.0.0.1", done));
  return server;
}
