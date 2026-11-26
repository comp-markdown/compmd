// The language server's preview page (docs/editor-support.md, decision 8): `@compmd/devtools`'s preview, full window,
// on the routes the server serves beside it, under the page's path with its token: `events`, the changes in the
// writer's editor; `request.json`, the document's request but its text, with the files and packs it reads;
// `registry/`, the proxy of the registries its configuration names; `files/`, the project's files, which the
// document's relative addresses (its images, its fonts) are read against, as on a site.

import { mountPreview } from "@compmd/devtools";
import type { Request } from "@compmd/processor";
import { sandbox } from "virtual:compmd";

/** The URL of the server's route `path`, beside the page. */
const route = (path: string): string => new URL(path, new URL(".", location.href)).href;

/** The document's request but its text, as the server gives it now, rendered at the reader's time. */
async function request(): Promise<Request> {
  const response = await fetch(route("request.json"));
  if (!response.ok) throw new Error(`request.json: ${String(response.status)} ${response.statusText}`);
  const given = (await response.json()) as Omit<Request, "now">;
  return { ...given, now: new Date().toISOString() };
}

/**
 * The page's base, the document's folder under `files/`: its relative addresses load from the project, as beside it on
 * a site. The page's own routes are read against its address, not its base, so they stay where they are.
 */
function baseAtFolderOf(path: string): void {
  const folders = path.split("/").slice(0, -1);
  const base = document.createElement("base");
  base.href = route(`files/${folders.map((folder) => `${encodeURIComponent(folder)}/`).join("")}`);
  document.head.prepend(base);
}

// The document is processed before its preview opens; if its request can't be had, the page still opens, and says why.
const opened = await request().catch(() => undefined);
if (opened) baseAtFolderOf(opened.path);

/** The frames' origin, which the server gives the page first in its head, as `createSandbox` reads it. */
const frames = (globalThis as { __COMPMD__?: { origin?: string } }).__COMPMD__?.origin ?? location.origin;

mountPreview(document.body, {
  sandbox,
  // The host's stylesheet, from the frames' origin, as a frame loads one.
  stylesheets: [`${frames}/document.css`],
  events: route("events"),
  request,
  // A pack the request doesn't hold yet, through the server's proxy of the configuration's registries.
  registries: { registries: { "*": route("registry/") }, fetch: "host" },
});
