// Serves a folder's files as GitHub Pages does: `index.html` for a folder, the type by extension, and
// `Access-Control-Allow-Origin: *`, which the playground's frame needs to load its scripts from its opaque origin. A
// request's path is read as the blog's dev server reads it (`@compmd/blog`), so one that can't be read is a `400`.
import { readFile, stat } from "node:fs/promises";
import type { IncomingMessage, ServerResponse } from "node:http";
import { join, normalize, sep } from "node:path";
import { requestedPath, sendBadRequest, sendFile } from "@compmd/blog";

/** The file `path` (decoded) names in `root`: `index.html` for a folder; `undefined` outside `root` or for none. */
export async function fileOf(root: string, path: string): Promise<string | undefined> {
  const file = normalize(join(root, path));
  if (file !== root && !file.startsWith(root + sep)) return undefined;
  const found = await stat(file).catch(() => undefined);
  if (found?.isDirectory()) return fileOf(root, join(path, "index.html"));
  return found?.isFile() ? file : undefined;
}

/**
 * Answers a request for a path under `prefix` with the file it names in `root`, a path that can't be read with a
 * `400`; calls `next` for any other path, or a file that isn't there.
 */
export function serveFolder(prefix: string, root: string) {
  const answer = async (request: IncomingMessage, response: ServerResponse): Promise<boolean> => {
    const path = requestedPath(request);
    if (path === undefined) {
      sendBadRequest(response, "The address can't be read.");
      return true;
    }
    const file = path.startsWith(prefix) ? await fileOf(root, path.slice(prefix.length)) : undefined;
    if (!file) return false;
    sendFile(response, file, await readFile(file), { "access-control-allow-origin": "*" });
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
