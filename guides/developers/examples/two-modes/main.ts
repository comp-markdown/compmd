import { init, OpenDocument } from "@compmd/processor";
import { createSandbox, type View } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./status.comp.md?raw";

await init();

let inputs: Record<string, unknown> = {};
const open = () =>
  new OpenDocument({ ...request, path: "status.comp.md", files: { "status.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = open();

// The same document twice: in one strict frame, and in this page with a frame per component use.
const views: View[] = (["frame", "host"] as const).map((page) => {
  const container = document.getElementById(page) as HTMLElement;
  const view = createSandbox({ ...sandbox, page }).open(container, { components: opened.components });
  view.on("error", ({ component, code, message }) => {
    console.warn(`${component}: ${code ?? "no diagnostic"}, ${message}`);
  });
  view.render(opened.expanded);
  return view;
});

// An input set in either view opens the document again, and both show it.
for (const view of views) {
  view.on("set", ({ input, value }) => {
    inputs = { ...inputs, [input]: value };
    opened.free();
    opened = open();
    for (const each of views) each.render(opened.expanded);
  });
}
