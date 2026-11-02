// The test page: `demo.comp.md`, processed with the pack as a host gives it, and rendered in host mode, where each
// diagram runs in a strict frame of its own, sized to it. The document's diagnostics show under it.
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./demo.comp.md?raw";

await init();
const opened = new OpenDocument({ ...request, path: "demo.comp.md", files: { "demo.comp.md": text }, now: new Date().toISOString() });
const page = document.getElementById("page");
const list = document.getElementById("diagnostics");
if (page && list) {
  const view = createSandbox({ ...sandbox, page: "host" }).open(page, { components: opened.components });
  view.render(opened.expanded);
  list.replaceChildren(
    ...opened.diagnostics.map(({ code, message }) => {
      const item = document.createElement("li");
      item.textContent = `${code}: ${message}`;
      return item;
    }),
  );
}
