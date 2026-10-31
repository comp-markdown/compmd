import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

await init();

// The document, open with what the app gives it: its packs, from compmd.config.json.
let inputs: Record<string, unknown> = {};
const open = () =>
  new OpenDocument({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = open();

// Its page, in a sandboxed frame.
const page = document.getElementById("page") as HTMLElement;
const view = createSandbox(sandbox).open(page, { components: opened.components });
view.render(opened.expanded);

// A reader changed an input: open the document again with its new value.
view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  opened = open();
  view.render(opened.expanded);
});
