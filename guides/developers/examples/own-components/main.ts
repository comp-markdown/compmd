import { init, OpenDocument } from "@compmd/processor";
import { createSandbox, type HostApi } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import text from "./notes.comp.md?raw";

await init();

// The app's data, which only the app's page reads: a document can't, and a component asks for it through `host`.
const summaries = new Map([["The Left Hand of Darkness", { count: 12, average: 4.2 }]]);

// What permissive components of packs given `host` may call. Their arguments come from documents: check them.
const host: HostApi = {
  reviewsOf: (book) => (typeof book === "string" ? (summaries.get(book) ?? null) : null),
};

let inputs: Record<string, unknown> = {};
const open = () =>
  new OpenDocument({ ...request, path: "notes.comp.md", files: { "notes.comp.md": text }, now: new Date().toISOString(), inputs });
let opened = open();

// Host mode: the page in this page, `stars` and `spoiler` inline, `reviews` in a permissive frame of its own.
const page = document.getElementById("page") as HTMLElement;
const view = createSandbox({ ...sandbox, page: "host", host }).open(page, { components: opened.components });
view.render(opened.expanded);

view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  opened.free();
  opened = open();
  view.render(opened.expanded);
});
