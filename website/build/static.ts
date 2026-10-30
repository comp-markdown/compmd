// Serves a folder's files as GitHub Pages does: `index.html` for a folder, the type by extension, and
// `Access-Control-Allow-Origin: *`, which the playground's frame needs to load its scripts from its opaque origin.
import { readFile, stat } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import { extname, join, normalize, sep } from "node:path";

const TYPES: Record<string, string> = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".json": "application/json",
  ".svg": "image/svg+xml",
  ".png": "image/png",
  ".wasm": "application/wasm",
  ".woff2": "font/woff2",
};

/** The file `path` names in `root`: `index.html` for a folder; `undefined` outside `root` or for none. */
export async function fileOf(root: string, path: string): Promise<string | undefined> {
  const file = normalize(join(root, decodeURIComponent(path)));
  if (file !== root && !file.startsWith(root + sep)) return undefined;
  const found = await stat(file).catch(() => undefined);
  if (found?.isDirectory()) return fileOf(root, join(path, "index.html"));
  return found?.isFile() ? file : undefined;
}

/**
 * Answers a request for a path under `prefix` with the file it names in `root`; calls `next` for any other path, or a
 * file that isn't there.
 */
export function serveFolder(prefix: string, root: string) {
  const answer = async (request: IncomingMessage, response: ServerResponse): Promise<boolean> => {
    const path = new URL(request.url ?? "/", "http://localhost").pathname;
    const file = path.startsWith(prefix) ? await fileOf(root, path.slice(prefix.length)) : undefined;
    if (!file) return false;
    const body = await readFile(file);
    response.setHeader("Content-Type", TYPES[extname(file)] ?? "application/octet-stream");
    response.setHeader("Access-Control-Allow-Origin", "*");
    response.end(body);
    return true;
  };
  return (request: IncomingMessage, response: ServerResponse, next: () => void): void => {
    answer(request, response).then(
      (answered) => {
        if (!answered) next();
      },
      () => {
        next();
      },
    );
  };
}
