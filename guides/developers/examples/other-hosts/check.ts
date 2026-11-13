// The hosting check, in the browser the app's readers use: the server serves the bundle as the sandbox needs.
import { runCheck } from "@compmd/sandbox/check";
import type { App } from "./page.ts";

const app = (await (await fetch("/app.json")).json()) as App;

// The Mermaid pack's module, which must load by its URL and draw.
const [place = ""] = Object.keys(app.modules);
const at = place.lastIndexOf("@");
const [version = "", ...path] = place.slice(at + 1).split("/");
const mermaid = { pack: place.slice(0, at), version, path: path.join("/") };

const report = await runCheck({
  container: document.getElementById("check") as HTMLElement,
  origin: "/compmd/",
  frames: app.frames,
  modules: ({ pack, version, path }) => app.modules[`${pack}@${version}/${path}`],
  packs: [{ place: mermaid, params: { code: "graph LR; A --> B" } }],
});
(document.getElementById("report") as HTMLElement).textContent = report.message;
document.body.dataset.checked = report.ok ? "passed" : "failed";
