import { init, OpenDocument, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";

await init();

// What the server gives every document, and the document itself.
const app = (await (await fetch("/app.json")).json()) as { request: Omit<Request, "path" | "now">; modules: Record<string, string> };
const text = await (await fetch("/plan.comp.md")).text();
const opened = new OpenDocument({ ...app.request, path: "plan.comp.md", files: { "plan.comp.md": text }, now: new Date().toISOString() });

// A built module loads in its frame from the server, checked against its file's hash.
const sandbox = createSandbox({
  base: "/compmd/",
  modules: ({ pack, version, path }) => {
    const place = `${pack}@${version}/${path}`;
    const integrity = app.modules[place];
    return integrity === undefined ? undefined : { url: new URL(`/compmd/packs/${place}`, location.href).href, integrity };
  },
});
const view = sandbox.open(document.getElementById("page") as HTMLElement, { components: opened.components });
view.render(opened.expanded);
