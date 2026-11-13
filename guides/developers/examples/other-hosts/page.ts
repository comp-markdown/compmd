import { init, OpenDocument, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";

await init();

/** What the server gives every page: the request's packs and policy, and where the bundle's frames and modules are. */
export interface App {
  request: Omit<Request, "path" | "now">;
  frames: string;
  modules: Record<string, { path: string; integrity: string }>;
}

const app = (await (await fetch("/app.json")).json()) as App;
const text = await (await fetch("/plan.comp.md")).text();
const opened = new OpenDocument({ ...app.request, path: "plan.comp.md", files: { "plan.comp.md": text }, now: new Date().toISOString() });

// The bundle is served under `compmd/` on this page's origin. A built module loads in its frame by its path there,
// checked against its file's hash.
const sandbox = createSandbox({
  origin: "/compmd/",
  frames: app.frames,
  modules: ({ pack, version, path }) => app.modules[`${pack}@${version}/${path}`],
});
const view = sandbox.open(document.getElementById("page") as HTMLElement, { components: opened.components });
view.render(opened.expanded);
