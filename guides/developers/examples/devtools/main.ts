import { init, OpenDocument, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

await init();

let inputs: Record<string, unknown> = {};
const requestOf = (): Request => ({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = new OpenDocument(requestOf());
const view = createSandbox(sandbox).open(document.getElementById("page") as HTMLElement, { components: opened.components });

/** In development, the inspector on the page's view, in a panel under it: loaded only then, so a build leaves it out. */
async function inspect() {
  const { mountDevtools } = await import("@compmd/devtools");
  const panel = document.getElementById("inspector") as HTMLElement;
  panel.style.height = "60vh";
  return mountDevtools(panel, { view, document: { request: requestOf() } });
}
// The dev server's mode: `DEV` follows `NODE_ENV`, which a build run by a test runner may set to `test`.
const inspector = import.meta.env.MODE === "development" ? await inspect() : undefined;

view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  const processed = requestOf();
  opened = new OpenDocument(processed);
  view.render(opened.expanded);
  inspector?.open({ request: processed });
});
view.render(opened.expanded);
