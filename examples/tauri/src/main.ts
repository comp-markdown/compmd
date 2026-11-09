// The page: the document the app processed, rendered by `@compmd/sandbox` in host mode. With `tauri-plugin-compmd`'s
// `init()`, the sandbox needs no `base`: the plugin gave the page the frames'. With `--check` or `--check-broken`, it
// checks what it drew and tells the app (`check.ts`).

import { invoke } from "@tauri-apps/api/core";
import { createSandbox, type Sandbox, type View, type ViewEvents } from "@compmd/sandbox";
// The Mermaid pack's built module, given to its frames as code: an app with the pack from npm imports it the same way.
import mermaid from "@comp-markdown/mermaid/dist/mermaid.js?raw";
import { check, type Mode } from "./check.ts";

/** What the app's `document` command gives: the processed document, in the JSON the sandbox reads. */
interface Processed {
  tree: Parameters<View["render"]>[0];
  hostComponents: Parameters<Sandbox["open"]>[1]["components"];
  diagnostics: unknown[];
}

const mode = await invoke<Mode>("mode");
const processed = await invoke<Processed>("document");
const container = document.querySelector<HTMLElement>("#document");
if (!container) throw new Error("the page has no #document");

// An app's own build of the frames (`frames_dir`) has no `base` on the page: it asks the plugin for it.
const base = mode === "check-broken" ? ((await invoke<string | null>("plugin:compmd|base")) ?? undefined) : undefined;
const sandbox = createSandbox({
  ...(base === undefined ? {} : { base }),
  page: "host",
  modules: (module) => (module.pack === "@comp-markdown/mermaid" ? { code: new Blob([mermaid], { type: "text/javascript" }) } : undefined),
});
const view = sandbox.open(container, { components: processed.hostComponents });
const observation = view.observe(() => undefined);
const errors: ViewEvents["error"][] = [];
const sets: ViewEvents["set"][] = [];
view.on("error", (error) => {
  errors.push(error);
  console.error(`CompMD: ${error.component}: ${error.message}`);
});
view.on("set", (set) => sets.push(set));
view.render(processed.tree);

if (mode !== "show") {
  const report = await check(mode, { view, container, observation, errors, sets, diagnostics: processed.diagnostics });
  const status = document.querySelector("#status");
  if (status) status.textContent = report.message;
  await invoke("report", { ok: report.ok, message: report.message });
}
