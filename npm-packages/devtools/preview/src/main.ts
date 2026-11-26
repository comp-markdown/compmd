// The language server's preview page (docs/editor-support.md, decision 8): `@compmd/devtools`'s preview, full window,
// on the routes the server serves beside it, under the page's path with its token: `events`, the changes in the
// writer's editor; `request.json`, the document's request but its text; `registry/`, the packs it imports.

import { mountPreview } from "@compmd/devtools";
import type { Request } from "@compmd/processor";
import { sandbox } from "virtual:compmd";

/** The URL of the server's route `path`, beside the page. */
const route = (path: string): string => new URL(path, new URL(".", location.href)).href;

/** The document's request but its text, as the server gives it now. */
async function request(): Promise<Request> {
  const response = await fetch(route("request.json"));
  if (!response.ok) throw new Error(`request.json: ${String(response.status)} ${response.statusText}`);
  return (await response.json()) as Request;
}

mountPreview(document.body, {
  sandbox,
  events: route("events"),
  request,
  // Every pack through the server, which holds the project's and fetches the others as the configuration says.
  registries: { registries: { "*": route("registry/") }, fetch: "host" },
});
