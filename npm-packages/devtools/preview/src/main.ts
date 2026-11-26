// The language server's preview page (docs/editor-support.md, decision 8): `@compmd/devtools`'s preview, full window,
// on the routes the server serves beside it, under the page's path with its token: `events`, the changes in the
// writer's editor; `request.json`, the document's request but its text, with the files and packs it reads;
// `registry/`, the proxy of the registries its configuration names.

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

mountPreview(document.body, {
  sandbox,
  events: route("events"),
  request,
  // A pack the request doesn't hold yet, through the server's proxy of the configuration's registries.
  registries: { registries: { "*": route("registry/") }, fetch: "host" },
});
