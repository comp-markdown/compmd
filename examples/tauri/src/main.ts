// The page: the document the app processed, rendered by `@compmd/sandbox` in host mode, with what the page's own build
// gives (`virtual:compmd`'s `sandbox`: the bundle's frames and modules) under the origin the plugin gives the page. With
// `--check` or `--check-broken`, it checks what it drew and tells the app (`check.ts`).

import { invoke } from "@tauri-apps/api/core";
import { createSandbox, type Sandbox, type View, type ViewEvents } from "@compmd/sandbox";
import { sandbox } from "virtual:compmd";
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
const hosting = document.querySelector<HTMLElement>("#hosting");
if (!container || !hosting) throw new Error("the page has no #document or #hosting");

const view = createSandbox({ ...sandbox, page: "host" }).open(container, { components: processed.hostComponents });
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
  const drawn = { view, container, hosting, observation, errors, sets, diagnostics: processed.diagnostics, components: processed.hostComponents };
  const report = await check(mode, drawn);
  const status = document.querySelector("#status");
  if (status) status.textContent = report.message;
  await invoke("report", { ok: report.ok, message: report.message });
}
