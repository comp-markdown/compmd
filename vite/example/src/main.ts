// The example's page: a document processed with the app's packs, rendered in host mode, so the app's own `stars`
// mounts in this page and every other component in a frame of its level. A star's click sets the input, and the
// document is processed again with it.
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import notes from "../notes.comp.md?raw";

await init();
let inputs: Record<string, unknown> = {};
const open = () => new OpenDocument({ ...request, path: "notes.comp.md", files: { "notes.comp.md": notes }, now: new Date().toISOString(), inputs });
let document_ = open();
const view = createSandbox({ ...sandbox, page: "host" }).open(document.getElementById("page")!, { components: document_.components });

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

view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  document_.free();
  document_ = open();
  view.render(document_.expanded);
});
view.render(document_.expanded);
report();
