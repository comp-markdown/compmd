// The example's page: a document processed with the app's packs, rendered in host mode, so the app's own `stars`
// mounts in this page and every other component in a frame of its level (`?page=frame` renders it in one strict frame
// instead). A star's click sets the input, and the document is processed again with it. In development, the Developer
// Tools' inspector shows what the page does, under it.
import { init, OpenDocument, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import notes from "../notes.comp.md?raw";

await init();
let inputs: Record<string, unknown> = {};
/** The document as the app processes it, with the inputs set so far. */
const requestOf = (): Request => ({ ...request, path: "notes.comp.md", files: { "notes.comp.md": notes }, now: new Date().toISOString(), inputs });
let document_ = new OpenDocument(requestOf());
const page = new URLSearchParams(location.search).get("page") === "frame" ? "frame" : "host";
const view = createSandbox({ ...sandbox, page }).open(document.getElementById("page")!, { components: document_.components });

/** Lists the document's diagnostics under the page. */
const report = () => {
  document.getElementById("diagnostics")!.replaceChildren(
    ...document_.diagnostics.map(({ code, message }) => {
      const item = document.createElement("li");
      item.textContent = `${code}: ${message}`;
      return item;
    }),
  );
};

/** In development, the inspector on the page's view, in a panel under it: loaded only then, so a build leaves it out. */
async function inspect() {
  const { mountDevtools } = await import("@compmd/devtools");
  const panel = document.getElementById("inspector")!;
  panel.style.height = "60vh";
  return mountDevtools(panel, { view, document: { request: requestOf() } });
}
// The dev server's mode: `DEV` follows `NODE_ENV`, which a build run by a test runner may set to `test`.
const inspector = import.meta.env.MODE === "development" ? await inspect() : undefined;

view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  document_.free();
  const processed = requestOf();
  document_ = new OpenDocument(processed);
  view.render(document_.expanded);
  inspector?.open({ request: processed });
});
view.render(document_.expanded);
report();
